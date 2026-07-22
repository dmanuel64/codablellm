import "globals.just"

cli *args:
    {{ cargo }} run -- {{ args }}

ui:
    {{ cargo }} tauri dev
