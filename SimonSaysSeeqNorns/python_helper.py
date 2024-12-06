import sys


# To include some CO2 PPM data for yesterday. 

def write_co2_ppm(folder):

    import requests

    from datetime import datetime
    from datetime import date
    from datetime import timedelta


    today = date.today()
    yesterday = today - timedelta(days = 1)

    url_for_daily_co2_ppm = 'https://gml.noaa.gov/webdata/ccgg/trends/co2/co2_daily_mlo.csv'


    #folder = '/home/we/dust/data/SimonSaysSeeqNorns/'
    file_name_for_latest_daily_co2_ppm = 'simon_says_seeq_web_data_co2_ppm_gml_noaa_gov_ccgg_daily_latest.csv'
    file_name_for_all_daily_co2_ppm = 'simon_says_seeq_web_data_co2_ppm_gml_noaa_gov_ccgg_all_daily.csv'


    x = requests.get(url_for_daily_co2_ppm)
    print(x.status_code)
    print(x.text)


    co2_ppm_yesterday_finder = "%s,%s,%s" %(yesterday.day, yesterday.month, yesterday.month) # use yesterday because data is (at least?) a day behind.

    co2_ppm_first_row_finder = "%s,%s,%s" %(1974, 5, 19) # first record was on this day.  




    print(co2_ppm_yesterday_finder)


    data_start_position = x.text.find(co2_ppm_first_row_finder)
    print (data_start_position)
    co2_ppm_data = x.text[data_start_position:]


    all_daily_co2_ppm_path = "%s%s" %(folder, file_name_for_all_daily_co2_ppm)
    f = open(all_daily_co2_ppm_path, "w")
    f.write(co2_ppm_data)
    f.close()


    print("I wrote the data %s up to the day %s to the file %s" %(x.text, co2_ppm_yesterday_finder, all_daily_co2_ppm_path))



    yesterday_start_position = co2_ppm_data.find(co2_ppm_yesterday_finder)

    print (yesterday_start_position)




    latest_line = x.text[yesterday_start_position:]

    print (latest_line)

    latest_value =  latest_line.split(",")[-1]

    print (latest_value)

    latest_daily_co2_ppm_path = "%s%s" %(folder, file_name_for_latest_daily_co2_ppm)


    f = open(latest_daily_co2_ppm_path, "w")
    f.write(latest_value)
    f.close()

    print("I wrote the value %s for the day %s to the file %s" %(latest_value, co2_ppm_yesterday_finder, latest_daily_co2_ppm_path))

    ################



    return latest_value




if __name__ == "__main__":
  folder = sys.argv[1]
  print(write_co2_ppm(folder))



