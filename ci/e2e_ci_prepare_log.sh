function finish {
    cp -r ~/.rainbow/logs .
}
trap finish ERR
trap finish EXIT