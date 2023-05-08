REQUIRED_PKG="lotus"



if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "[$(date)] -- Detected Linux"
        # ...
    REQUIRED_PKG="some-package"
    PKG_OK=$(dpkg-query -W --showformat='${Status}\n' $REQUIRED_PKG|grep "install ok installed")
    echo "[$(date)] -- Checking for $REQUIRED_PKG"
    if [ "" = "$PKG_OK" ]; then
        echo "No $REQUIRED_PKG. Setting up $REQUIRED_PKG."
        sudo apt-get --yes install hwloc
        wget https://github.com/filecoin-project/lotus/releases/download/v1.19.0/lotus_1.19.0_linux_amd64.tar.gz
        tar -xvf lotus_1.19.0_linux_amd64.tar.gz
        sudo mv lotus_1.19.0_linux_amd64/lotus /usr/local/bin/lotus
    fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
    echo "[$(date)] -- Detected Mac OSX"
        # Mac OSX
    PKG_OK=$(brew list lotus)
    echo "[$(date)] -- Checking for $REQUIRED_PKG"
    if [ "" = "$PKG_OK" ]; then
        echo "[$(date)] -- No $REQUIRED_PKG. Setting up $REQUIRED_PKG."
        brew tap filecoin-project/lotus
        brew install lotus
    fi
else 
    echo "Unsupported OS"
    exit 1
fi

echo "[$(date)] -- Starting lotus lite daemon."
FULLNODE_API_INFO="wss://wss.node.glif.io/apigw/lotus" nohup lotus daemon --lite > node.log 2>&1 &
echo "[$(date)] -- Node pid: $!. Check node.log for logs if an issue arises."   
echo "[$(date)] -- Started daemon. Waiting 5 seconds for it to finish setting up."
sleep 5

read -p "[$(date)] -- Do you wish to import a private key? [y/n] " yn
case $yn in
    [Yy]* ) lotus wallet import; break;;
    [Nn]* ) exit;;
    * ) echo "Please answer yes or no.";;
esac
