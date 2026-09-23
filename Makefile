CONTAINER_NAME=swagger-api-kh

.PHONY: build-release fmt fmt-check test test-all-features coverage

ifeq ($(shell uname),Linux)
  OPEN=xdg-open
else
  OPEN=open
endif

ifeq ($(strip $(TARGET)),)
build-release:
	$(error TARGET must be set to a Rust target triple)
else ifeq ($(TARGET),aarch64-unknown-linux-gnu)
build-release:
	cross build --locked --release --target $(TARGET)
else
build-release:
	cargo build --locked --release --target $(TARGET)
endif

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

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

test:
	cargo test --locked

test-all-features:
	cargo test --locked --all-features

coverage:
	cargo llvm-cov --locked --ignore-filename-regex '(^|/)tests/' --html --fail-under-lines 98
	cargo llvm-cov report --ignore-filename-regex '(^|/)tests/' --json --output-path target/llvm-cov/coverage.json
