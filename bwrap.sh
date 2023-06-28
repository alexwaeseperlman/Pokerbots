bwrap \
	--ro-bind /usr /usr \
	--dir /tmp \
	--dir /var \
	--bind /root /root \
	--proc /proc \
	--dev /dev \
	--ro-bind /etc/resolv.conf /etc/resolv.conf \
	--ro-bind /lib /lib \
	--ro-bind /lib64 /lib64 \
	--ro-bind /usr/bin /usr/bin \
	--ro-bind /bin /bin \
	--ro-bind /usr/sbin /usr/sbin \
	--unshare-all \
	--die-with-parent \
	--setenv PS1 "secured-shell (hopefully)$ " \
	/bin/bash
