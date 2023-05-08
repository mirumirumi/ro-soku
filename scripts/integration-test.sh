results=()

execute_tests() {
    echo "Starting tests for $1"
    shift
    "$@"
    local result=$?
    results+=($result)
    return $result
}

execute_tests "Binance spot" bash ./scripts/exchanges/binance/spot.sh
execute_tests "Binance perpetual" bash ./scripts/exchanges/binance/perpetual.sh

execute_tests "bitbank spot" bash ./scripts/exchanges/bitbank/spot.sh

execute_tests "bitmex perpetual" bash ./scripts/exchanges/bitmex/perpetual.sh

execute_tests "Bybit spot" bash ./scripts/exchanges/bybit/spot.sh
execute_tests "Bybit perpetual" bash ./scripts/exchanges/bybit/perpetual.sh

execute_tests "OKX spot" bash ./scripts/exchanges/okx/spot.sh
execute_tests "OKX perpetual" bash ./scripts/exchanges/okx/perpetual.sh

all_successful=true
for result in "${results[@]}"; do
    if [ "$result" -ne 0 ]; then
        all_successful=false
        break
    fi
done

if $all_successful; then
    echo "✅ Final result: Succeeded!"
else
    echo "❌ Final result: Failed."
    exit 1
fi
