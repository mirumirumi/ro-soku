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
    --exchange binance \
    --type perpetual \
    --symbol BTC/USDT \
    --interval 1min \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick t \
    --order desc \
    --format csv

execute_command \
    --exchange binance \
    --type perpetual \
    --symbol BTC/USDT \
    --interval 3min \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick t \
    --order desc \
    --format csv

execute_command \
    --exchange binance \
    --type perpetual \
    --symbol BTC/USDT \
    --interval 5min \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick t \
    --order desc \
    --format csv

execute_command \
    --exchange binance \
    --type perpetual \
    --symbol BTC/USDT \
    --interval 15min \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick t \
    --order desc \
    --format csv

execute_command \
    --exchange binance \
    --type perpetual \
    --symbol BTC/USDT \
    --interval 30min \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick t \
    --order desc \
    --format csv

execute_command \
    --exchange binance \
    --type perpetual \
    --symbol BTC/USDT \
    --interval 1hour \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick t \
    --order desc \
    --format csv

execute_command \
    --exchange binance \
    --type perpetual \
    --symbol BTC/USDT \
    --interval 2hour \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick t \
    --order desc \
    --format csv

execute_command \
    --exchange binance \
    --type perpetual \
    --symbol BTC/USDT \
    --interval 4hour \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick t \
    --order desc \
    --format csv

execute_command \
    --exchange binance \
    --type perpetual \
    --symbol BTC/USDT \
    --interval 6hour \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick t \
    --order desc \
    --format csv

execute_command \
    --exchange binance \
    --type perpetual \
    --symbol BTC/USDT \
    --interval 8hour \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick t \
    --order desc \
    --format csv

execute_command \
    --exchange binance \
    --type perpetual \
    --symbol BTC/USDT \
    --interval 12hour \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick t \
    --order desc \
    --format csv

execute_command \
    --exchange binance \
    --type perpetual \
    --symbol BTC/USDT \
    --interval 1day \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick t \
    --order desc \
    --format csv

execute_command \
    --exchange binance \
    --type perpetual \
    --symbol BTC/USDT \
    --interval 3day \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick t \
    --order desc \
    --format csv

execute_command \
    --exchange binance \
    --type perpetual \
    --symbol BTC/USDT \
    --interval 1week \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick t \
    --order desc \
    --format csv

execute_command \
    --exchange binance \
    --type perpetual \
    --symbol BTC/USDT \
    --interval 1month \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick t \
    --order desc \
    --format csv

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
