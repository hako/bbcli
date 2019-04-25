import os
import json
import arrow
import requests

BBC_URL = "https://www.bbc.co.uk"
BBC_POLLING_URL = "https://polling.bbc.co.uk"
API_BASE_URL = "https://trevor-producer-cdn.api.bbci.co.uk"

class BBC():

    def get_top_stories(self):
        news = self.get_bbc_story()
        if news == None:
            return None
        else:
            data = json.dumps(news.json())
            return self.parse_news(data)

    def get_ticker(self):
        ticker = self.get_bbc_ticker()
        if ticker == None:
            return None
        else:
            data = ticker.json(strict=False)
        return self.parse_ticker_data(data)

    def parse_ticker_data(self, data):
        tickers = []

        if bool(data["asset"]) == False:
            return tickers

        # Headline
        headline = data["asset"]["headline"]

        # News Link as in /news/
        url = data["asset"]["assetUri"]

        ticker = Ticker(headline, "BREAKING NEWS", "true", BBC_URL + url)
        tickers.append(ticker)
        return tickers

    def parse_news(self, stories):
        t_news = []
        ts_section = ""
        data = json.loads(stories)
        for _, d in enumerate(data['relations']):
            for rel in d['content']['relations']:
                if(rel['content']['type'] != "bbc.mobile.news.collection"):
                    pass
                else:
                    ts_section = rel['content']['name']
            ts_title = d['content']['name']
            timestamp = d['content']['lastUpdated']
            ts_time = arrow.get(float(timestamp) / 1000).humanize()
            ts_subtext  = "Last updated: " + str(ts_time) + " | " + str(ts_section)
            ts_link = str(d['content']['shareUrl'])
            news = News(ts_title, ts_link, ts_subtext)
            t_news.append(news)
        return t_news

    def get_bbc_story(self):
        res = None
        headers = {
            'User-Agent': 'BBCNews/5.6.0.100 UK (Pixel 4; Android 6.0)', 
            'Accept-Encoding': 'gzip',
            'Connection': 'Keep-Alive',
            'Accept': 'application/json'
        }
        try:
            res = requests.get(API_BASE_URL + "/content/cps/news/front_page", data=None, headers=headers)
        except requests.ConnectionError as e:
            if hasattr(e, 'reason'):
                print ('We failed to reach a server.')
                print ('Reason: ', e.reason)
            elif hasattr(e, 'code'):
                print ('The server couldn\'t fulfill the request.')
                print ('Error code: ', e.code)
        return res

    def get_bbc_ticker(self):
        res = None
        ua = {
           'User-Agent': 'Mozilla/5.0 (Windows NT 6.2) AppleWebKit/5311 (KHTML, like Gecko) Chrome/13.0.837.0 Safari/5311'
        }
        try:
            res = requests.get(BBC_POLLING_URL + "/news/latest_breaking_news_waf?audience=Domestic", data=None, headers=ua)
        except requests.ConnectionError as e:
            if hasattr(e, 'reason'):
                print('We failed to reach a server.')
                print('Reason: ', e.reason)
            elif hasattr(e, 'code'):
                print('The server couldn\'t fulfill the request.')
                print('Error code: ', e.code)
        return res

class News():

    def __init__(self, title, link, subtext):
        self.title = title
        self.link = link
        self.subtext = subtext


class Ticker():

    def __init__(self, headline, prompt, breaking, url):
        self.headline = headline
        self.prompt = prompt
        self.breaking = breaking
        self.url = url
