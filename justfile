# === titrate === #

### default command (comes first) ###

default:
    just --choose

# FIXME(kate): this does not inject linkerd, fix that.
run: cluster-delete cluster-create cluster-image-import cluster-apply
    
# === cluster management === #

cluster-name := "titrate"
client-image-archive := "titrate-client.tar"
server-image-archive := "titrate-server.tar"
# TODO(kate):

cluster-create:
    k3d cluster create {{ cluster-name }}

cluster-delete:
    k3d cluster delete {{ cluster-name }}

cluster-stop:
    k3d cluster stop {{ cluster-name }}

# NB: this does not inject linkerd, do this from the linkerd repo.
cluster-apply:
    kubectl apply -f k8s/client.yml    
    kubectl apply -f k8s/server.yml    

cluster-get-client-pods:
    kubectl --namespace titrate get pods -l app=titrate-client -o wide

cluster-get-client-logs:
    kubectl --namespace titrate logs -l app=titrate-client

cluster-get-server-pods:
    kubectl --namespace titrate get pods -l app=titrate-server -o wide

cluster-get-server-logs:
    kubectl --namespace titrate logs -l app=titrate-server

cluster-get-logs:
    kubectl --namespace titrate logs -l app=titrate-server --all-containers | tail -20
    echo
    kubectl --namespace titrate logs -l app=titrate-client --all-containers | tail -20

cluster-image-import: cluster-image-import-client cluster-image-import-server
cluster-image-import-client: image-save-client
    k3d image import {{ client-image-archive }} --cluster {{ cluster-name }}
cluster-image-import-server: image-save-server
    k3d image import {{ server-image-archive }} --cluster {{ cluster-name }}

cluster-redeploy-client: cluster-image-import-client
  kubectl --namespace titrate rollout restart deployment client-1
cluster-redeploy-server: cluster-image-import-server
  kubectl --namespace titrate rollout restart deployment server-1

# === measurement commands === #

# records the amount of cpu being used by each server's proxy container.
server-cpu-file := "server-cpu-report.csv"
record-server-cpu-usage:
    just get-server-cpu-usage >> {{ server-cpu-file }}
get-server-cpu-usage:
    kubectl --namespace titrate top pod -l app=titrate-server \
      --no-headers --containers | grep linkerd-proxy \
      | tr -s ' ' | sed -e 's/ $//g' | sed -e 's/ /,/g'

# records the amount of cpu being used by each client's proxy container.
client-cpu-file := "client-cpu-report.csv"
record-client-cpu-usage:
    just get-client-cpu-usage >> {{ client-cpu-file }}
get-client-cpu-usage:
    kubectl --namespace titrate top pod -l app=titrate-client \
      --no-headers --containers | grep linkerd-proxy \
      | tr -s ' ' | sed -e 's/ $//g' | sed -e 's/ /,/g'

# === run docker images === #

image-run-client:
    docker run titrate-client

image-run-server:
    docker run titrate-server

# === build docker images === #

image-build: image-build-client image-build-server
image-build-client:
    docker build --tag titrate-client --file Dockerfile.client .
image-build-server:
    docker build --tag titrate-server --file Dockerfile.server .

# === save docker images === #

image-save: image-save-client image-save-server
image-save-client: image-build-client
    docker save -o {{ client-image-archive }} titrate-client
image-save-server: image-build-server
    docker save -o {{ server-image-archive }} titrate-server

# === run binaries === #

run-client:
    cargo run --bin titrate-client

run-server:
    cargo run --bin titrate-server

# === dev commands === #

build:
    cargo build --all-features --all-targets

check:
    cargo check --all-features --all-targets

check-short:
    cargo check --all-features --all-targets --message-format=short

doc:
    cargo doc

doc-open:
    cargo doc --open

doc-test:
    cargo test --doc

lint:
    cargo clippy --all-targets

test:
    cargo nextest run --all-features --all-targets

test-all: test doc-test

# === ci: build, document, test, and lint

ci: build doc test-all lint

# === watch command output === #

watch-check:
    cargo watch --clear --why --shell 'just check'

watch-check-short:
    cargo watch --clear --why --shell 'just check-short'

watch-test:
    cargo watch --clear --why --shell 'just test'

watch-ci:
    cargo watch --clear --why --shell 'just ci'
