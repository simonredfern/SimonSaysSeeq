# SimonSaysSeeq on Bela Salt 

See this video for an intro into some features and how to modify source code: [SimonSaysSeeq on Bela Salt Twitch video](https://www.twitch.tv/videos/885185134)

![Jan 2025 SimonSaysSeeq - on Bela Salt with Salt+](https://github.com/user-attachments/assets/7a6c7f39-5683-410d-90df-77e93d651988)


Various notes:

Analog outputs (code value from 0 to 1) corresponding to the range 0 to 5V


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


To enable network for bela to mac:

```

vi /etc/network/interfaces

systemctl restart networking.service

ping 8.8.8.8

root@bela:~# cat /etc/resolv.conf 
nameserver 192.168.3.1
nameserver 8.8.8.8

```

*Experiemental* notes about monome grids (currently not used or working with the Bela here) (See SimonSaysSeeqNorns folder instead for monome):

https://github.com/padenot/bela-utils/blob/master/bela-setup-monome.sh

Follow the instructions (including various hacks to files)
https://forum.bela.io/d/240-monome-grid-bela/24

Install all that is required to use a monome device on a vanilla bela board,
start the serialosc daemon on boot using systemd.
Requires an internet connection to use apt and git.


```

sudo apt-get update -y
sudo apt-get install -y libudev-dev



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

```

Learn to code to serialosc

http://daniel-bytes.github.io/serialosc_example/



========


// TO Understand render see the example in Fundamentals: minimal/render.cpp

// In general, see https://www.youtube.com/watch?v=XJ2fFqGexCM


// So Bela can get to the internet via a Mac with ethernet over USB

// ssh root@bela.local
// vi /etc/network/interfaces
// auto usb0
// iface usb0 inet dhcp
// auto usb1
// iface usb1 inet dhcp
// And enable Mac OS Sharing like this:
// Mac OS Preferences. Sharing From = Wifi. (drop down list) To Computers using = Bela (check box). Internet Sharing = Yes. (ticked)  
// then on Bela: systemctl restart networking.service
// Login again and
// ping 8.8.8.8




// Note: Bela might run out of disk space
// du -h --max-depth=1
// #!/bin/bash
// dir=/path/to/directory/you/want/to/check
// num=[number of top largest directories to list)
// du -ah $dir | sort -n -r | head -n $num
// du -hs /var/*

// Can apparently delete /var/cache/apt


// Add this to your /etc/init.d

// root@bela:/etc/init.d# cat bela_startup.sh 
// #!/bin/bash
// rm -f /var/log/*.log || true
// echo > /var/log/syslog
// rm -f /var/log/*.gz || true
// echo > /var/log/syslog.1
// echo $(date -u) "I ran /etc/init.d/bela_startup.sh on startup" >> /var/log/bela_startup.log.keep


// echo > /var/log/*.log


// Make it executable with 
// chmod u+x bela_startup.sh
// chmod +x bela_startup.sh <- Need this else it doesn't run.

// add it to crontab (edited via, for example, crontab -e)
// not sure how successfully this runs
// @reboot sleep 60 && /etc/init.d/bela_startup.sh



//root@bela:/var/log# rm /var/log/*.log
//root@bela:/var/log# rm /var/log/syslog
//root@bela:/var/log# rm /var/log/*.gz
//root@bela:/var/log# rm /var/log/syslog.1 


==========

// To find midi ports Bela can see, type "amidi -l" in the Bela command line.
// Also  lsusb -t via ssh 

/*
root@bela:~/bin# cat log_usb.sh
#!/bin/bash
echo Hello
amidi -l
lsusb -t
*/

/*
root@bela:~/bin/SimonSaysSeeq# amidi -l
Dir Device    Name
IO  hw:0,0    f_midi <-- This is the connection to your computer (device port on Bela)
IO  hw:1,0,0  USB MIDI Interface MIDI 1 <-- This is the device connected to USB host port on the Bela
*/

// NOTE: It seems there is a timing / loading issue with the USB midi port...
// In order for USB host midi port to work, either: 
// 1) save two copies of this patch under loop_A and loop_B, set the settings to start with loop_* and after booting the Salt, press the left button on Salt for more than two seconds 
// or 
// 2) Save this patch via the IDE after the USB cables are plugged in.
  

//const char* gMidiPort0 = "hw:0,0"; // This is the computer via USB cable







==========



////////////////////////////////////////////////


// const uint8_t BRIGHT_0 = 0;
// const uint8_t BRIGHT_1 = 10;
// const uint8_t BRIGHT_2 = 20;
// const uint8_t BRIGHT_3 = 75;
// const uint8_t BRIGHT_4 = 100;
// const uint8_t BRIGHT_5 = 255;


=======


// "Ghost notes" are created to cancel out a note-off in keyboard_midi_note_events that is created  during the note off of low velocity notes.
// class GhostNote
// {
//  public:
//    uint8_t tick_count_in_sequence = 0;
//    uint8_t is_active = 0;
// };

//GhostNote channel_x_ghost_events[128];

////////////////////////////////////////
// Bit Constants for bit wise operations 




