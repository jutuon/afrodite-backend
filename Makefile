
CARGO_CRATE_ARGS = 	-p api_internal \
					-p config \
					-p database \
					-p model \
					-p server \
					-p server_common \
					-p server_api \
					-p server_data \
					-p test_mode \
					-p test_mode_macro \
					-p utils \
					-p simple_backend \
					-p simple_backend_utils \
					-p simple_backend_model \
					-p simple_backend_config \
					-p simple_backend_database \
					-p simple_backend_image_process \
					-p pihka-backend

ifdef CONTINUE_FROM
TEST_QA_ARGS = --continue-from $(CONTINUE_FROM)
endif

TMP_FILE = ./target/tmp_file_for_makefile

# Default rule
run:
	RUST_LOG=$${RUST_LOG:-info} cargo run --bin pihka-backend

run-release:
	RUST_LOG=$${RUST_LOG:-info} cargo run --bin pihka-backend --release

fmt:
	cargo +nightly fmt $(CARGO_CRATE_ARGS)
fix:
	cargo fix ${CARGO_CRATE_ARGS}
test:
	RUST_LOG=info cargo run --bin pihka-backend -- --sqlite-in-ram test ${TEST_ARGS} qa ${TEST_QA_ARGS}
unit-test:
	mkdir -p database/sqlite/current
	DATABASE_URL="sqlite:database/sqlite/current/current.db" cargo test

update-manager-submodule:
	git submodule update --remote --merge
update-api-bindings:
	cargo build --bin pihka-backend
	./target/debug/pihka-backend open-api > $(TMP_FILE)
	openapi-generator-cli generate \
	-i $(TMP_FILE) \
	-g rust \
	-o crates/api_client \
	--package-name api_client
# Workarounds for generator bugs
# Command output is redirected as macOS sed doesn't support normal -i
	sed 's/software_options: SoftwareOptions/software_options: crate::models::SoftwareOptions/g' crates/api_client/src/apis/common_admin_api.rs > $(TMP_FILE)
	cp $(TMP_FILE) crates/api_client/src/apis/common_admin_api.rs
	sed 's/queue: ModerationQueueType/queue: crate::models::ModerationQueueType/g' crates/api_client/src/apis/media_admin_api.rs > $(TMP_FILE)
	cp $(TMP_FILE) crates/api_client/src/apis/media_admin_api.rs
	sed 's/content_type: MediaContentType/content_type: crate::models::MediaContentType/g' crates/api_client/src/apis/media_api.rs > $(TMP_FILE)
	cp $(TMP_FILE) crates/api_client/src/apis/media_api.rs

validate-openapi:
	cargo build --bin pihka-backend
	./target/debug/pihka-backend open-api > $(TMP_FILE)
	openapi-generator-cli validate \
	-i $(TMP_FILE)

migrations-run:
	mkdir -p database/sqlite/current
	DATABASE_URL="database/sqlite/current/current.db" diesel migration run
reset-database:
	mkdir -p database/sqlite/current
	DATABASE_URL="database/sqlite/current/current.db" diesel database reset

profile-build:
	RUSTC_BOOTSTRAP=1 RUSTFLAGS=-Zself-profile=target/profile-build cargo build --bin pihka-backend

code-stats:
	@/bin/echo -n "Lines:"
	@find \
	crates/api_internal \
	crates/config \
	crates/database \
	crates/model \
	crates/server \
	crates/server_common \
	crates/server_api \
	crates/server_data \
	crates/test_mode \
	crates/test_mode_macro \
	crates/utils \
	crates/simple_backend \
	crates/simple_backend_utils \
	crates/simple_backend_model \
	crates/simple_backend_config \
	crates/simple_backend_database \
	crates/simple_backend_image_process \
	crates/pihka-backend \
	-name '*.rs' | xargs wc -l | tail -n 1
	@echo "\nCommits:   `git rev-list --count HEAD` total"
