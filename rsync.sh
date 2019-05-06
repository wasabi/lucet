set -Eeuxo pipefail

cd "$(dirname ${BASH_SOURCE[0]})"

rsync --exclude='.git/' --exclude='target' -avz -e 'docker-machine ssh circleci-docker-machine' . :/home/ubuntu/lucet

while true; do
  fswatch --exclude "\.git.*" -1 -r .
  rsync --exclude='.git/' --exclude='target' -avz -e 'docker-machine ssh circleci-docker-machine' . :/home/ubuntu/lucet
done

