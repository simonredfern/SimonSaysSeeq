# SimonSaysSeeq on Bela Salt 

See this video for an intro into some features and how to modify source code: [SimonSaysSeeq on Bela Salt Twitch video](https://www.twitch.tv/videos/885185134)

![SimonSaysSeeq on Bela Salt](https://user-images.githubusercontent.com/485218/130356331-5ea7c7bd-a3ff-4046-b6a7-ee596cb29795.png)


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
