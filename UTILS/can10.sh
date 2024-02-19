sudo ip link set can0 down
sudo ip link set can0 up type can bitrate 1000000
cantools monitor TER.dbc -c can0
