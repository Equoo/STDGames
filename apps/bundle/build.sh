docker rmi sshfs-bundle-builder
docker build -f Dockerfile.build --target builder -t sshfs-bundle-builder .
docker create --name tmp-sshfs sshfs-bundle-builder
docker cp tmp-sshfs:/bundle ./sshfs-bundle
docker rm tmp-sshfs
