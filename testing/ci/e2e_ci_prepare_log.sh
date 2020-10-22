function finish {
    cp -r ~/.rainbow/logs .
    cp ~/.pm2/pm2.log .
}
trap finish ERR
trap finish EXIT