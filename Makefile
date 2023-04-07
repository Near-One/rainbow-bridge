include .env

nixify:
	yarn2nix > nix/yarn.nix

image-load:
	nix build && ./result | docker load

image-remote-load:
	nix build '.#production-image-stream' && ./result | ssh ${DEPLOY_URL} docker load
