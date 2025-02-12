image=gusto48786/gimme-3d

build:
	docker build --platform linux/amd64 -t $(image) .

push:
	docker build --platform linux/amd64 -t $(image):latest --push .

upload-gltf:
	aws s3 cp ./glb/ s3://jq-staging-matko/gltf/ --recursive --profile jq-staging-sysops

run-server: build
	docker run --memory=1024m \
		--cpus=1 \
		-it --rm \
		-p 3030:3030 \
		-v $$(pwd)/config.toml:/app/config.toml \
		$(image)

build-client:
	cd client && go build

run-client-webp:
	cd client && ./client -all -save -size=2000 -iformat=webp -oformat=webp

run-client:
	cd client && ./client -all -save -size=2400

vegeta:
	vegeta attack -targets=request.txt -format=http -duration=20s -timeout=60s -rate=2 \
	| tee results.bin \
	| vegeta report

renderer-bash:
	docker run -it --rm -v $(PWD):/app -w /app --entrypoint="" $(image) bash

linux-bash:
	docker run -it --rm -v $(PWD):/app -w /app rust:1.75.0-bookworm bash

request:
	curl -X POST \
	-H "Content-Type: application/json" \
	-d @request.json http://localhost:3030/render \
	-o output.png

convert-assets:
	cargo run --bin cmd -- convert -i ./fbx -b -o glb

preview:
	RUST_BACKTRACE=1 cargo run --release --bin preview

serve:
	RUST_BACKTRACE=1 cargo run --release --bin cmd -- serve

helm-install:
	helm upgrade gimme-3d ./helm/gimme-3d --values ./helm/gimme-3d/values.yaml --install
