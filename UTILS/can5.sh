sudo ip link set can0 down
sudo ip link set can0 up type can bitrate 500000
cantools monitor INVERTER.dbc -c can0
