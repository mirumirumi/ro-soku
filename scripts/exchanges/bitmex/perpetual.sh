ro-soku() {
    ./target/release/ro-soku_x86_64_linux "$@"
}

results=()

execute_command() {
    ro-soku "$@"
    results+=($?)
    sleep 3
}

execute_command \
    --exchange bitmex \
    --type perpetual \
    --symbol BTC/USDT \
    --interval 1min \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick h \
    --order desc \
    --format json

execute_command \
    --exchange bitmex \
    --type perpetual \
    --symbol BTC/USDT \
    --interval 5min \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick h \
    --order desc \
    --format json

execute_command \
    --exchange bitmex \
    --type perpetual \
    --symbol BTC/USDT \
    --interval 1hour \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick h \
    --order desc \
    --format json

execute_command \
    --exchange bitmex \
    --type perpetual \
    --symbol BTC/USDT \
    --interval 1day \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick h \
    --order desc \
    --format json

all_successful=true
for result in "${results[@]}"; do
    if [ "$result" -ne 0 ]; then
        all_successful=false
        break
    fi
done

if $all_successful; then
    echo "✅ Succeeded!"
else
    echo "❌ Failed."
    exit 1
fi
