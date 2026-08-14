import re

PATH = r'e:\DEV\Project\OpenStream Engine\package\openwrt\files\streamproxyd.init'

with open(PATH, 'r', encoding='utf-8', newline='') as f:
    text = f.read()

# Determine line ending
lf = '\r\n' if '\r\n' in text else '\n'

# Step 1: remove the dangerous uci confdir block
# Pattern: from the line after mkdir until fi (including blank lines around it)
pattern = re.compile(
    r'\tmkdir -p /etc/dnsmasq\.d /tmp/dnsmasq\.d\n'
    r'\n'
    r'\tif uci -q get dhcp\.@dnsmasq\[0\] >/dev/null 2>&1; then\n'
    r'\t\tif ! uci -q get dhcp\.@dnsmasq\[0\]\.confdir \| grep -q "/etc/dnsmasq\.d"; then\n'
    r"\t\t\tuci -q add_list dhcp\.@dnsmasq\[0\]\.confdir='/etc/dnsmasq\.d'\n"
    r'\t\t\tuci -q commit dhcp\n'
    r'\t\tfi\n'
    r'\tfi'
)

replacement1 = (
    "\t# Пишем конфиг в /tmp/dnsmasq.d/ (основное, загружается автоматически)\n"
    "\t# и /etc/dnsmasq.d/ (персистентно). НЕ трогаем UCI dhcp confdir:\n"
    "\t# uci add_list объединяет пути пробелом => dnsmasq падает с\n"
    "\t# 'No such file or directory' для несуществующего пути '/tmp/dnsmasq.d /etc/dnsmasq.d'.\n"
    "\tmkdir -p /tmp/dnsmasq.d /etc/dnsmasq.d"
)

m = pattern.search(text)
if m:
    text = text[:m.start()] + replacement1 + text[m.end():]
    print("Step 1 OK: removed uci add_list confdir block")
else:
    # Try normalizing: replace \r\n with \n in search
    text_n = text.replace('\r\n', '\n')
    m = pattern.search(text_n)
    if m:
        text_n = text_n[:m.start()] + replacement1 + text_n[m.end():]
        text = text_n
        print("Step 1 OK (after normalizing line endings)")
    else:
        print("Step 1 FAIL")
        idx = text.find('mkdir -p /etc/dnsmasq.d')
        print(repr(text[max(0,idx-2):idx+400]))

# Step 2: swap output target - write to /tmp first, then cp to /etc
old2 = '} > /etc/dnsmasq.d/openstream.conf\n\n\tcp -f /etc/dnsmasq.d/openstream.conf /tmp/dnsmasq.d/openstream.conf 2>/dev/null || true'
new2 = '} > /tmp/dnsmasq.d/openstream.conf\n\n\tcp -f /tmp/dnsmasq.d/openstream.conf /etc/dnsmasq.d/openstream.conf 2>/dev/null || true'

if old2 in text:
    text = text.replace(old2, new2)
    print("Step 2 OK: changed output order /tmp -> /etc")
else:
    print("Step 2 FAIL")
    idx = text.find('/etc/dnsmasq.d/openstream.conf')
    print(repr(text[max(0,idx-20):idx+150]))

with open(PATH, 'w', encoding='utf-8', newline='') as f:
    f.write(text)
print("Done - file written")
