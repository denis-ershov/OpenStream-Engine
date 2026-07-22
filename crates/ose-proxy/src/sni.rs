//! Минимальный разбор TLS ClientHello → SNI (без полной TLS-библиотеки).

/// Извлекает server_name из ClientHello (байт 0 = Handshake / 0x16).
pub fn extract_sni(data: &[u8]) -> Option<String> {
    if data.len() < 5 || data[0] != 0x16 {
        return None;
    }
    // TLS record: type(1) ver(2) len(2) + handshake
    let record_len = u16::from_be_bytes([data[3], data[4]]) as usize;
    if data.len() < 5 + record_len {
        // неполный record — всё равно пробуем то, что есть
    }
    let hs = &data[5..];
    if hs.len() < 4 || hs[0] != 0x01 {
        return None; // не ClientHello
    }
    let hs_len = ((hs[1] as usize) << 16) | ((hs[2] as usize) << 8) | (hs[3] as usize);
    if hs.len() < 4 + hs_len.min(hs.len().saturating_sub(4)) {
        // partial ok
    }
    let mut p = &hs[4..];
    // client_version(2) + random(32)
    if p.len() < 34 {
        return None;
    }
    p = &p[34..];
    // session_id
    if p.is_empty() {
        return None;
    }
    let sid_len = p[0] as usize;
    p = p.get(1 + sid_len..)?;
    // cipher_suites
    if p.len() < 2 {
        return None;
    }
    let cs_len = u16::from_be_bytes([p[0], p[1]]) as usize;
    p = p.get(2 + cs_len..)?;
    // compression
    if p.is_empty() {
        return None;
    }
    let comp_len = p[0] as usize;
    p = p.get(1 + comp_len..)?;
    // extensions
    if p.len() < 2 {
        return None;
    }
    let ext_len = u16::from_be_bytes([p[0], p[1]]) as usize;
    p = p.get(2..)?;
    let mut exts = &p[..ext_len.min(p.len())];
    while exts.len() >= 4 {
        let typ = u16::from_be_bytes([exts[0], exts[1]]);
        let len = u16::from_be_bytes([exts[2], exts[3]]) as usize;
        let body = exts.get(4..4 + len)?;
        if typ == 0 {
            // server_name
            return parse_server_name_list(body);
        }
        exts = &exts[4 + len..];
    }
    None
}

fn parse_server_name_list(body: &[u8]) -> Option<String> {
    if body.len() < 2 {
        return None;
    }
    let list_len = u16::from_be_bytes([body[0], body[1]]) as usize;
    let mut p = body.get(2..2 + list_len.min(body.len().saturating_sub(2)))?;
    while p.len() >= 3 {
        let name_type = p[0];
        let name_len = u16::from_be_bytes([p[1], p[2]]) as usize;
        let name = p.get(3..3 + name_len)?;
        if name_type == 0 {
            return String::from_utf8(name.to_vec()).ok();
        }
        p = &p[3 + name_len..];
    }
    None
}

/// Сколько байт ещё нужно прочитать, чтобы record был полным (грубо).
pub fn tls_record_total_len(data: &[u8]) -> Option<usize> {
    if data.len() < 5 || data[0] != 0x16 {
        return None;
    }
    let record_len = u16::from_be_bytes([data[3], data[4]]) as usize;
    Some(5 + record_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Минимальный ClientHello с SNI=example.com (синтетический).
    fn hello_with_sni(sni: &str) -> Vec<u8> {
        let sni_bytes = sni.as_bytes();
        let mut server_name = Vec::new();
        server_name.extend_from_slice(&(sni_bytes.len() as u16 + 3).to_be_bytes()); // list len
        server_name.push(0); // host_name
        server_name.extend_from_slice(&(sni_bytes.len() as u16).to_be_bytes());
        server_name.extend_from_slice(sni_bytes);

        let mut extensions = Vec::new();
        extensions.extend_from_slice(&0u16.to_be_bytes()); // type server_name
        extensions.extend_from_slice(&(server_name.len() as u16).to_be_bytes());
        extensions.extend_from_slice(&server_name);

        let mut body = Vec::new();
        body.extend_from_slice(&[0x03, 0x03]); // version
        body.extend_from_slice(&[0u8; 32]); // random
        body.push(0); // session_id len
        body.extend_from_slice(&2u16.to_be_bytes()); // cipher len
        body.extend_from_slice(&[0x00, 0x2f]); // TLS_RSA_WITH_AES_128_CBC_SHA
        body.push(1); // compression len
        body.push(0); // null
        body.extend_from_slice(&(extensions.len() as u16).to_be_bytes());
        body.extend_from_slice(&extensions);

        let mut hs = Vec::new();
        hs.push(0x01); // ClientHello
        let bl = body.len();
        hs.push(((bl >> 16) & 0xff) as u8);
        hs.push(((bl >> 8) & 0xff) as u8);
        hs.push((bl & 0xff) as u8);
        hs.extend_from_slice(&body);

        let mut record = Vec::new();
        record.push(0x16);
        record.extend_from_slice(&[0x03, 0x01]);
        record.extend_from_slice(&(hs.len() as u16).to_be_bytes());
        record.extend_from_slice(&hs);
        record
    }

    #[test]
    fn extracts_sni() {
        let data = hello_with_sni("playlist.ttvnw.net");
        assert_eq!(
            extract_sni(&data).as_deref(),
            Some("playlist.ttvnw.net")
        );
    }

    #[test]
    fn rejects_http() {
        assert!(extract_sni(b"CONNECT example.com:443 HTTP/1.1\r\n").is_none());
    }
}
