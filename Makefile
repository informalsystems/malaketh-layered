all: clean
	cargo build
	cargo run --bin malachitebft-eth-utils genesis
	docker compose up -d
	./scripts/add_peers.sh 
	cargo run --bin malachitebft-eth-app -- testnet --nodes 3 --home nodes
	echo 👉 Grafana dashboard is available at http://localhost:3000
	bash scripts/spawn.bash --nodes 3 --home nodes

clean:
	docker compose down
	rm -rf ./nodes
	rm -rf ./rethdata
	rm -rf ./monitoring/data-grafana

