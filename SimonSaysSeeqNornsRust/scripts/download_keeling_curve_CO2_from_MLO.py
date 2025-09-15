# A Python script to download parse and save some Atmospheric CO2 concentrations data from the Mauna Loa Observatory (MLO)
# The data is known as the The Keeling Curve
# This file was copied from SimonSaysSeeqNorns
import sys
import os


# To include some CO2 PPM data for yesterday.

def log_attempt(folder):
    """Log the datetime of the download attempt"""
    from datetime import datetime
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    log_path = os.path.join(folder, "last_CO2_download_attempt.log")
    with open(log_path, "w") as f:
        f.write(timestamp)

def log_success(folder):
    """Log the datetime of successful download"""
    from datetime import datetime
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    log_path = os.path.join(folder, "last_CO2_download_success.log")
    with open(log_path, "w") as f:
        f.write(timestamp)

def log_exception(folder, exception):
    """Log the last exception"""
    from datetime import datetime
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    log_path = os.path.join(folder, "last_CO2_download_exception.log")
    with open(log_path, "w") as f:
        f.write(f"{timestamp}: {str(exception)}")

def write_co2_ppm(folder):

    import requests
    import time

    from datetime import datetime
    from datetime import date
    from datetime import timedelta
    
    # Log the attempt
    log_attempt(folder)


    # We need to know yesterdays date becuase data on the website is generally one day old.
    today = date.today()
    yesterday = today - timedelta(days = 1)

    url_for_daily_co2_ppm = 'https://gml.noaa.gov/webdata/ccgg/trends/co2/co2_daily_mlo.csv'


    #folder = '/home/we/dust/data/SimonSaysSeeqNorns/'
    file_name_for_latest_daily_co2_ppm = 'simon_says_seeq_web_data_co2_ppm_gml_noaa_gov_ccgg_daily_latest.csv'
    file_name_for_all_daily_co2_ppm = 'simon_says_seeq_web_data_co2_ppm_gml_noaa_gov_ccgg_all_daily.csv'


    # Retry logic for network requests
    max_retries = 5
    base_delay = 30  # Start with 30 seconds
    
    for attempt in range(max_retries):
        try:
            print(f"Attempt {attempt + 1}/{max_retries} to download CO2 data...")
            x = requests.get(url_for_daily_co2_ppm, timeout=30)
            print(x.status_code)
            print(x.text)
            break  # Success, exit retry loop
        except requests.exceptions.RequestException as e:
            if attempt < max_retries - 1:
                delay = base_delay * (2 ** attempt)  # Exponential backoff
                print(f"Network error on attempt {attempt + 1}: {e}")
                print(f"Retrying in {delay} seconds...")
                time.sleep(delay)
                continue
            else:
                raise  # Re-raise the exception if all retries failed

    try:


        co2_ppm_yesterday_finder = "%s,%s,%s" %(yesterday.day, yesterday.month, yesterday.year) # use yesterday because data is (at least?) a day behind.

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

        # Only write latest daily file if we found yesterday's data
        if yesterday_start_position != -1:
            latest_line = x.text[yesterday_start_position:]

            print (latest_line)

            latest_value =  latest_line.split(",")[-1]

            print (latest_value)

            latest_daily_co2_ppm_path = "%s%s" %(folder, file_name_for_latest_daily_co2_ppm)

            f = open(latest_daily_co2_ppm_path, "w")
            f.write(latest_value)
            f.close()

            print("I wrote the value %s for the day %s to the file %s" %(latest_value, co2_ppm_yesterday_finder, latest_daily_co2_ppm_path))
        else:
            print("Could not find data for yesterday (%s), skipping latest daily file write" % co2_ppm_yesterday_finder)
            # Don't write to the latest daily file if we can't find yesterday's data
            latest_value = None

        ################

        # Log success if we got this far
        log_success(folder)

        return latest_value
    
    except Exception as e:
        print(f"Error downloading CO2 data: {e}")
        log_exception(folder, e)
        return None




if __name__ == "__main__":
  folder = sys.argv[1]
  print(write_co2_ppm(folder))
