ro-soku() {
    ./target/release/ro-soku "$@"
}

results=()

execute_command() {
    ro-soku "$@"
    results+=($?)
    sleep 3
}

execute_command \
    --exchange bitbank \
    --type spot \
    --symbol BTC/JPY \
    --interval 1min \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick o \
    --order asc \
    --format tsv

execute_command \
    --exchange bitbank \
    --type spot \
    --symbol BTC/JPY \
    --interval 5min \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick o \
    --order asc \
    --format tsv

execute_command \
    --exchange bitbank \
    --type spot \
    --symbol BTC/JPY \
    --interval 15min \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick o \
    --order asc \
    --format tsv

execute_command \
    --exchange bitbank \
    --type spot \
    --symbol BTC/JPY \
    --interval 30min \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick o \
    --order asc \
    --format tsv

execute_command \
    --exchange bitbank \
    --type spot \
    --symbol BTC/JPY \
    --interval 1hour \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick o \
    --order asc \
    --format tsv

execute_command \
    --exchange bitbank \
    --type spot \
    --symbol BTC/JPY \
    --interval 4hour \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick o \
    --order asc \
    --format tsv

execute_command \
    --exchange bitbank \
    --type spot \
    --symbol BTC/JPY \
    --interval 8hour \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick o \
    --order asc \
    --format tsv

execute_command \
    --exchange bitbank \
    --type spot \
    --symbol BTC/JPY \
    --interval 12hour \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick o \
    --order asc \
    --format tsv

execute_command \
    --exchange bitbank \
    --type spot \
    --symbol BTC/JPY \
    --interval 1day \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick o \
    --order asc \
    --format tsv

execute_command \
    --exchange bitbank \
    --type spot \
    --symbol BTC/JPY \
    --interval 1week \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick o \
    --order asc \
    --format tsv

execute_command \
    --exchange bitbank \
    --type spot \
    --symbol BTC/JPY \
    --interval 1month \
    --term-start 2023-01-10T23:00:00Z \
    --term-end 2023-01-12T00:00:00Z \
    --pick o \
    --order asc \
    --format tsv

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
