import itertools
from bs4 import BeautifulSoup
from urllib2 import Request, urlopen, URLError

BASE_URL = 'http://news.ycombinator.com'


class HN():

    def get_zipped_rows(self, soup):
        titles = soup.select('td.title')
        subtexts = soup.select('td.subtext')

        titles = [row for row in (itertools.islice(
            titles[1:], 0, None, 2)) if not row.text == 'More'
        ]
        subtexts = [row for row in subtexts]

        return zip(titles, subtexts)

    def build_story(self, all_rows):
        all_stories = []
        for (title, subtext) in all_rows:
            stitle = title.text
            slink = title.a['href']
            ssubtext = subtext.text
            story = Story(stitle, slink, ssubtext)
            all_stories.append(story)
        return all_stories

    def get_hn_source(self, url):
        headers = {'User-Agent': 'Mozilla/5.0'}
        req = Request(url, data=None, headers=headers)
        try:
            response = urlopen(req).read()
        except URLError as e:
            if hasattr(e, 'reason'):
                print 'We failed to reach a server.'
                print 'Reason: ', e.reason
            elif hasattr(e, 'code'):
                print 'The server couldn\'t fulfill the request.'
                print 'Error code: ', e.code
        soup = BeautifulSoup(response)
        return soup

    def get_top_stories(self):
        soup = self.get_hn_source(BASE_URL)
        all_rows = self.get_zipped_rows(soup)
        return self.build_story(all_rows)


class Story():

    def __init__(self, title, link, subtext):
        self.title = title
        self.link = link
        self.subtext = subtext
