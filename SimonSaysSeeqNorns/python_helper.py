import sys





# To include some CO2 PPM data for yesterday. 

def write_latest_co2_ppm(folder):

    import requests

    from datetime import datetime
    from datetime import date
    from datetime import timedelta


    today = date.today()
    yesterday = today - timedelta(days = 1)

    url_for_daily_co2_ppm = 'https://gml.noaa.gov/webdata/ccgg/trends/co2/co2_daily_mlo.csv'


    #folder = '/home/we/dust/data/SimonSaysSeeqNorns/'
    file_name_for_daily_co2_ppm = 'simon_says_seeq_web_data_co2_ppm_gml_noaa_gov_ccgg_daily_latest.csv'


    x = requests.get(url_for_daily_co2_ppm)
    print(x.status_code)
    print(x.text)


    co2_ppm_yesterday_finder = "%s,%s,%s" %(yesterday.day, yesterday.month, yesterday.month) # use yesterday because data is (at least?) a day behind.


    print(co2_ppm_yesterday_finder)

    pos = x.text.find(co2_ppm_yesterday_finder)

    print (pos)




    latest_line = x.text[pos:]

    print (latest_line)

    my_value =  latest_line.split(",")[-1]

    print (my_value)

    daily_co2_ppm_path = "%s%s" %(folder, file_name_for_daily_co2_ppm)


    f = open(daily_co2_ppm_path, "w")
    f.write(my_value)
    f.close()


    print("I wrote the value %s for the day %s to the file %s" %(my_value, co2_ppm_yesterday_finder, daily_co2_ppm_path))
    return my_value




if __name__ == "__main__":
  folder = sys.argv[1]
  print(write_latest_co2_ppm(folder))



