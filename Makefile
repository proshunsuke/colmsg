VERSION=`$(shell pwd)/target/release/colmsg -V | cut -b 8-`

release/x86_64-linux:
	cargo build --release
	tar -C target/release -czvf target/release/colmsg-v${VERSION}-x86_64-unknown-linux-gnu.tar.gz colmsg

release/x86_64-darwin:
	PATH="${HOME}/Documents/osxcross/target/bin:${PATH}" cargo build --release --target x86_64-apple-darwin
	tar -C target/x86_64-apple-darwin/release -czvf target/x86_64-apple-darwin/release/colmsg-v${VERSION}-x86_64-apple-darwin.tar.gz colmsg

release/x86_64-win:
	cross build --release --target x86_64-pc-windows-gnu
	@cd target/x86_64-pc-windows-gnu/release/ && zip colmsg-v${VERSION}-x86_64-pc-windows-gnu.zip colmsg.exe
