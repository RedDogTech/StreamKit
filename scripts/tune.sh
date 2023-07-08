#!/bin/bash

sysctl -w net.core.netdev_max_backlog=25000

# allow testing with buffers up to 128MB
sysctl -w net.core.rmem_max=134217728
sysctl -w net.core.rmem_default=134217728
sysctl -w net.core.wmem_max=134217728
sysctl -w net.core.wmem_default=134217728
sysctl -w net.core.optmem_max=134217728

# increase Linux autotuning TCP buffer limit to 64MB
sysctl -w net.ipv4.tcp_rmem=\"4096 87380 67108864\"
sysctl -w net.ipv4.tcp_wmem=\"4096 65536 67108864\"

# recommended default congestion control is htcp
sysctl -w net.ipv4.tcp_congestion_control=htcp

# recommended for hosts with jumbo frames enabled
sysctl -w net.ipv4.tcp_mtu_probing=1

# recommended for CentOS7+/Debian8+ hosts
sysctl -w net.core.default_qdisc=fq