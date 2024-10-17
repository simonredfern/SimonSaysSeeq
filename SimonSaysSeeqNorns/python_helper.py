# To include some CO2 PPM data for yesterday. 

import requests

from datetime import datetime

current_second = datetime.now().second
current_minute = datetime.now().minute
current_hour = datetime.now().hour

current_day = datetime.now().day
current_month = datetime.now().month
current_year = datetime.now().year



url_for_daily_co2_ppm = 'https://gml.noaa.gov/webdata/ccgg/trends/co2/co2_daily_mlo.csv'
file_name_for_daily_co2_ppm = 'simon_says_seeq_web_data_co2_ppm_gml_noaa_gov_ccgg_daily_latest.csv'


x = requests.get(url_for_daily_co2_ppm)
print(x.status_code)
print(x.text)


co2_ppm_today_finder = "%s,%s,%s" %(current_year, current_month, current_day - 1) # use yesterday because data is (at least?) a day behind.


print(co2_ppm_today_finder)

pos = x.text.find(co2_ppm_today_finder)

print (pos)




latest_line = x.text[pos:]

print (latest_line)

my_value =  latest_line.split(",")[-1]

print (my_value)


f = open(file_name_for_daily_co2_ppm, "w")
f.write(my_value)
f.close()


print("I wrote the value %s for the day %s to the file %s" %(my_value, co2_ppm_today_finder, file_name_for_daily_co2_ppm))

