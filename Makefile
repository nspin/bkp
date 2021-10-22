bin_name := bulk-primitive
target_bin := target/debug/$(bin_name)
libexec_bin := libexec/bulk/$(bin_name)

.PHONY: all
all: $(libexec_bin)

$(libexec_bin): $(target_bin)
	install -D -T $< $@

$(target_bin): build

.PHONY: build
build:
	cargo build
