.PHONY: build

PASSTHROUGH_ENV = \
    HOME=$(HOME) \
    DISPLAY=$(DISPLAY) \
    XAUTHORITY=$(XAUTHORITY) \
    PATH=$(PATH) \
    DBUS_SESSION_BUS_ADDRESS=$(DBUS_SESSION_BUS_ADDRESS) \
    XDG_RUNTIME_DIR=$(XDG_RUNTIME_DIR) \
    XDG_DATA_DIRS=$(XDG_DATA_DIRS) \
    SNAP=$(SNAP) \
    SNAP_NAME=$(SNAP_NAME) \
    SNAP_REVISION=$(SNAP_REVISION)

dev:
	env -i $(PASSTHROUGH_ENV) cargo tauri dev

build:
	env -i $(PASSTHROUGH_ENV) cargo tauri build

clean:
	rm -rf ./build ./src-tauri/target