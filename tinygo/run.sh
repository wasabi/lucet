set -Eeuxo pipefail

cd "$(dirname ${BASH_SOURCE[0]})"

cd ..

make install
/opt/lucet/bin/lucetc -o ./tinygo/main.so ./tinygo/main.wasm --bindings ./lucet-wasi/bindings.json
/opt/lucet/bin/lucet-wasi --entrypoint cwa_main tinygo/main.so

