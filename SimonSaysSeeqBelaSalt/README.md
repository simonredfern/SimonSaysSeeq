# SimonSaysSeeq on Bela Salt 

See this video for an intro into some features and how to modify source code: [SimonSaysSeeq on Bela Salt Twitch video](https://www.twitch.tv/videos/885185134)

![SimonSaysSeeq - on Bela Salt with Salt+ 24 Nov 2021](https://user-images.githubusercontent.com/485218/143320241-88c7fbc3-f101-4f58-8a5e-9443cc83ecad.png)

To find out which USB devices are active (if disconnected from your main computer)

First create SimonSaysSeeq directory in /var/log becasue we'll log there.

Then create the following file e.g. here /root/bin/SimonSaysSeeq/midi_usb_info.job:

```
#!/bin/bash
LOG_FILE="/var/log/SimonSaysSeeq/usb_midi_info_log"
echo Hello from midi_usb_info.job. >> $LOG_FILE
echo The date / time is: >> $LOG_FILE
echo `date` >> $LOG_FILE
echo Your MIDI / USB devices are: >> $LOG_FILE
echo From amidi -l >> $LOG_FILE
amidi -l >> $LOG_FILE
echo From lsusb -t >> $LOG_FILE
lsusb -t >> $LOG_FILE
echo Bye >> $LOG_FILE
echo ================================== >> $LOG_FILE
```


Make it executable
```
chmod +x /root/bin/SimonSaysSeeq/midi_usb_info.job
```

Test calling it:
```
cd /root/bin
./midi_usb_info.job
```

And to see the output
```
tail -f -n 100 /var/log/SimonSaysSeeq/usb_midi_info_log
```

You might see something like:
```
Hello from midi_usb_info.job.
The date / time is:
Mon Mar 8 07:41:56 UTC 2021
Your MIDI / USB devices are:
Dir Device    Name
IO  hw:0,0    f_midi
IO  hw:1,0,0  USB MIDI Interface MIDI 1
/:  Bus 01.Port 1: Dev 1, Class=root_hub, Driver=musb-hdrc/1p, 480M
    |__ Port 1: Dev 2, If 0, Class=Audio, Driver=snd-usb-audio, 12M
    |__ Port 1: Dev 2, If 1, Class=Audio, Driver=snd-usb-audio, 12M
Bye
==================================
```


And call it from cron
```
crontab -e
# m h  dom mon dow   command
* * * * * /root/bin/SimonSaysSeeq/midi_usb_info.job
```

To work with a monome grids:

https://github.com/padenot/bela-utils/blob/master/bela-setup-monome.sh


To enable network for bela to mac:

vi /etc/network/interfaces

systemctl restart networking.service


ping 8.8.8.8

root@bela:~# cat /etc/resolv.conf 
nameserver 192.168.3.1
nameserver 8.8.8.8

========

Follow the instructions (including various hacks to files)
https://forum.bela.io/d/240-monome-grid-bela/24

#!/bin/bash

# Install all that is required to use a monome device on a vanilla bela board,
# start the serialosc daemon on boot using systemd.
# Requires an internet connection to use apt and git.

(
sudo apt-get update -y

sudo apt-get install -y libudev-dev
)


sudo apt install libudev-dev liblo-dev libavahi-compat-libdnssd-dev 

git clone https://github.com/monome/libmonome.git
./waf configure
./waf
sudo ./waf install
cd ..

git clone https://github.com/monome/serialosc.git
cd serialosc
git submodule init
git submodule update
./waf configure
./waf
sudo ./waf install
cd ..

cat << EOF > serialoscd.service
[Unit]
Description=serialosc daemon
[Service]
Type=simple
ExecStart=/usr/local/bin/serialoscd
PIDFile=/var/run/serialoscd.pid
RemainAfterExit=no
Restart=on-failure
RestartSec=5s
[Install]
WantedBy=multi-user.target
EOF

chmod 777 serialoscd.service
mv serialoscd.service /lib/systemd/system/serialoscd.service
ln -s /lib/systemd/system/serialoscd.service /etc/systemd/system/multi-user.target.wants/serialoscd.service
ldconfig



now try: 

systemctl start serialoscd

Learn to code to serialosc

http://daniel-bytes.github.io/serialosc_example/
