import json
import datetime
import requests
import os

API_BASE_URL = "http://trevor-producer-cdn.api.bbci.co.uk"
BBC_URL = "http://www.bbc.co.uk"

class BBC():
    
    def get_top_stories(self):
        news = self.get_bbc_story()
        data = json.dumps(news.json())
        return self.parse_news(data)

    def get_ticker(self):
        ticker = self.get_bbc_ticker()
        data = json.dumps(ticker.json())
        return self.parse_ticker_data(data)

    def parse_ticker_data(self, ticker_data):
        tickers = []
        data = json.loads(ticker_data)
        for d in data['entries']:
            headline = d['headline']
            prompt = d['prompt']
            breaking = d['isBreaking']
            if 'url' in d:
                url = d['url']
            else:
                url = ""
            ticker = Ticker(headline, prompt, breaking, url)
            tickers.append(ticker)
        return tickers

    def parse_news(self, stories):
        tnews = []
        ts_section = ""
        data = json.loads(stories)
        for i, d in enumerate(data['relations']):
            for rel in d['content']['relations']:
                if(rel['content']['type'] != "bbc.mobile.news.collection"):
                    pass
                else:
                    ts_section = rel['content']['name']
            ts_title = d['content']['name']
            timestamp = str(d['content']['lastUpdated']).replace("000", "")
            ts_time = datetime.datetime.fromtimestamp(float(timestamp))
            ts_subtext  = "Last updated: " + str(ts_time) + " | " + str(ts_section)
            ts_link = str(d['content']['shareUrl'])
            news = News(ts_title, ts_link, ts_subtext)
            tnews.append(news)
        return tnews
    
    def get_bbc_story(self):
        headers = {
        'User-Agent': 'BBCNews/3.0.1 UK (Nexus 4; Android 5.0)', 
        'Accept-Encoding': 'gzip', 
        'Connection': 'Keep-Alive',
        'Accept': 'application/json'
        }
        try:
            res = requests.get(API_BASE_URL + "/content/cps/news/front_page", data=None, headers=headers)
        except requests.ConnectionError as e:
            if hasattr(e, 'reason'):
                print 'We failed to reach a server.'
                print 'Reason: ', e.reason
            elif hasattr(e, 'code'):
                print 'The server couldn\'t fulfill the request.'
                print 'Error code: ', e.code
        return res

    def get_bbc_ticker(self):
        ua = {
           'User-Agent': 'Mozilla/5.0 (Windows NT 6.2) AppleWebKit/5311 (KHTML, like Gecko) Chrome/13.0.837.0 Safari/5311'
        }
        try:
            res = requests.get(BBC_URL + "/news/10284448/ticker.sjson", data=None, headers=ua)
        except requests.ConnectionError as e:
            if hasattr(e, 'reason'):
                print 'We failed to reach a server.'
                print 'Reason: ', e.reason
            elif hasattr(e, 'code'):
                print 'The server couldn\'t fulfill the request.'
                print 'Error code: ', e.code
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