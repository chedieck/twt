install:
	cargo build
	cp target/debug/twt ${HOME}/.local/bin/twt
	sed 's|PATH_TO_EXECUTABLE|${HOME}/.local/bin/twt|' extra/twt.service.template > extra/twt.service
	cp -f extra/twt.service ${HOME}/.config/systemd/user/twt.service
	[[ -d ~/.config/twt ]] || mkdir ~/.config/twt
	[[ -f ~/.config/twt/config.toml ]] || cp extra/config.toml ${HOME}/.config/twt/
	systemctl --user enable twt
	systemctl --user start twt

install-nosystemd:
	cargo build
	cp target/debug/twt ${HOME}/.local/bin/twt


uninstall:
	rm ${HOME}/.local/bin/twt
	rm -rf ${HOME}/.local/share/twt
	systemctl --user stop twt
	systemctl --user disable twt
	rm ${HOME}/.config/systemd/user/twt.service

reinstall:
	systemctl --user stop twt
	make install
