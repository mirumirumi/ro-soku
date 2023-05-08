# act -W ./.github/workflows/cron-test.yaml

echo "Starting tests for Binance spot" && sh ./scripts/exchanges/binance/spot.sh
echo "Starting tests for Binance perpetual" && sh ./scripts/exchanges/binance/perpetual.sh

echo "Starting tests for bitbank spot" && sh ./scripts/exchanges/bitbank/spot.sh

echo "Starting tests for bitmex perpetual" && sh ./scripts/exchanges/bitmex/perpetual.sh

echo "Starting tests for Bybit spot" && sh ./scripts/exchanges/bybit/spot.sh
echo "Starting tests for Bybit perpetual" && sh ./scripts/exchanges/bybit/perpetual.sh

echo "Starting tests for OKX spot" && sh ./scripts/exchanges/okx/spot.sh
echo "Starting tests for OKX perpetual" && sh ./scripts/exchanges/okx/perpetual.sh

