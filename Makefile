VERSION=`$(shell pwd)/target/release/colmsg -V | cut -b 8-`
CONTAINER_NAME=swagger-api-kh

ifeq ($(shell uname),Linux)
  OPEN=xdg-open
else
  OPEN=open
endif

release/x86_64-linux:
	cargo build --release
	tar -C target/release -czvf target/release/colmsg-v${VERSION}-x86_64-unknown-linux-gnu.tar.gz colmsg

release/x86_64-darwin:
	corss build --release --target x86_64-apple-darwin
	tar -C target/x86_64-apple-darwin/release -czvf target/x86_64-apple-darwin/release/colmsg-v${VERSION}-x86_64-apple-darwin.tar.gz colmsg

release/aarch64-darwin:
	cross build --release --target aarch64-apple-darwin
	tar -C target/aarch64-apple-darwin/release -czvf target/aarch64-apple-darwin/release/colmsg-v${VERSION}-aarch64-apple-darwin.tar.gz colmsg

release/x86_64-win:
	cross build --release --target x86_64-pc-windows-gnu
	@cd target/x86_64-pc-windows-gnu/release/ && zip colmsg-v${VERSION}-x86_64-pc-windows-gnu.zip colmsg.exe

server/kh:
	docker-compose up swagger-api-kh

stop/server/kh:
	docker-compose stop swagger-api-kh

restart/server/kh:
	docker-compose restart swagger-api-kh

start/ui-editor/kh:
	docker-compose up -d swagger-editor-kh swagger-ui-kh

stop/ui-editor/kh:
	docker-compose stop swagger-editor-kh swagger-ui-kh

restart/ui-editor/kh:
	docker-compose restart swagger-editor-kh swagger-ui-kh

open/editor/kh:
	$(OPEN) http://localhost:8001/

open/ui/kh:
	$(OPEN) http://localhost:8002/

server/n46:
	docker-compose up swagger-api-n46

stop/server/n46:
	docker-compose stop swagger-api-n46

restart/server/n46:
	docker-compose restart swagger-api-n46

start/ui-editor/n46:
	docker-compose up -d swagger-editor-n46 swagger-ui-n46

stop/ui-editor/n46:
	docker-compose stop swagger-editor-n46 swagger-ui-n46

restart/ui-editor/n46:
	docker-compose restart swagger-editor-n46 swagger-ui-n46

open/editor/n46:
	$(OPEN) http://localhost:8004/

open/ui/n46:
	$(OPEN) http://localhost:8005/

down:
	docker-compose down --rmi all --volumes --remove-orphans

ssh:
	docker exec -it $(CONTAINER_NAME) /bin/sh
