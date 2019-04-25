import os
import time
import urwid
import webbrowser
import configparser

from bbcapi import BBC
from datetime import datetime

_config = None

class BBCNews(object):

    def __init__(self, i, story):
        self.index = i + 1
        self.story = story

    @property
    def story_number(self):
        index = str(self.index)
        if len(index) == 1:
            return ''.join((' ', index))
        return self.index

    @property
    def story_title(self):
        return self.story.title

    @property
    def story_link(self):
        return self.story.link

    @property
    def story_subtext(self):
        return self.story.subtext


def get_top_stories():
    bbc = BBC()
    news = bbc.get_top_stories()
    if news == None:
        pass
    else:
        for i, story in enumerate(news[:30]):
            yield BBCNews(i, story)


def read_config():
    filename = 'bbcli'
    config = configparser.ConfigParser()
    if os.path.exists(os.path.expanduser('~' + '/.' + filename)):
        config.read(os.path.expanduser('~' + '/.' + filename))
    elif os.path.exists ( os.path.expanduser('~' + '/.config/' + filename)):
        config.read(os.path.expanduser('~' + '/.config/' + filename))
    return config


def open_browser(url):
    webbrowser.open(url, 2)


class ItemWidget(urwid.WidgetWrap):

    def __init__(self, s):
        self.story_link = s.story_link
        story_title = urwid.AttrWrap(urwid.Text(
            '%s. %s' % (s.story_number, s.story_title)),
            'body', 'focus'
        )
        story_subtext = urwid.AttrWrap(urwid.Text(
            '    %s' % (s.story_subtext)),
            'subtext', 'focus'
        )
        pile = urwid.Pile([story_title, story_subtext])
        self.item = [
            urwid.Padding(pile, left=1, right=1),
            ('flow', urwid.AttrWrap(urwid.Text(
                ' ', align="right"), 'body', 'focus'
            ))
        ]
        w = urwid.Columns(self.item, focus_column=0)
        super(ItemWidget, self).__init__(w)

    def selectable(self):
        return True

    def keypress(self, size, key):
        return key


class UI(object):

    palette = [
        ('head', '', '', '', '#FFF', '#E00'),
        ('body', '', '', '', '#000', '#FFF'),
        ('offline', '', '', '', '#FFF', '#000'),
        ('offline_bg', '', '', '', '#FFF', '#000'),
        ('footer', '', '', '', '#000', 'g89'),
        ('focus', '', '', '', '#FFF', 'dark red'),
        ('subtext', '', '', '', 'g55', '#FFF'),
        ('breaking', '', '', '', '#FFF', '#E00'),
    ]

    header = [
        urwid.AttrWrap(urwid.Text(
            ' BBC | NEWS', align='center'), 'head'
        ),
        ('flow', urwid.AttrWrap(urwid.Text(
            ' ', align="left"), 'head'
        )),
    ]
    header = urwid.Columns(header)

    offlineHeader = [
        urwid.AttrWrap(urwid.Text(
            ' BBC | NEWS (Offline)', align='center'), 'head'
        ),
        ('flow', urwid.AttrWrap(urwid.Text(
            ' ', align="left"), 'head'
        )),
    ]
    offlineHeader = urwid.Columns(offlineHeader)

    # defaults
    keys = {
        'quit'        : 'q',
        'open'        : 'w',
        'tabopen'     : 't',
        'refresh'     : 'r',
        'latest'      : 'l',
        'scroll_up'   : 'k',
        'scroll_down' : 'j',
        'top'         : 'g',
        'bottom'      : 'G'
    }

    mouse_button = {
        'left'       : 1,
        'middle'     : 2,
        'right'      : 3,
        'wheel_up'   : 4,
        'wheel_down' : 5
    }

    tickers = None
    ticker_count = -1
    count = 2
    link = ""

    def run(self):
        self.make_screen()
        urwid.set_encoding('utf-8')
        self.set_keys()
        try:
            self.loop.run()
        except KeyboardInterrupt:
            print("Keyboard interrupt received, quitting gracefully")
            raise urwid.ExitMainLoop

    def make_screen(self):
        self.view = urwid.Frame(
            urwid.AttrWrap(self.populate_stories(), 'body'),
            header=self.header
        )

        self.loop = urwid.MainLoop(
            self.view,
            self.palette,
            unhandled_input=self.handle_user_input
        )
        self.loop.screen.set_terminal_properties(colors=256)
        self.update_ticker()
        self.loop.set_alarm_in(200, self._wrapped_refresh)
        self.loop.set_alarm_in(5, self.next_item)

    def set_keys(self):
        global _config
        _config = read_config()
        if _config.has_section('Keys'):
            for option in _config.options('Keys'):
                try:
                    self.keys[option] = _config.get('Keys', option)
                except:
                    pass

    def get_stories(self):
        items = list()
        for story in get_top_stories():
            items.append(ItemWidget(story))
        return items

    def isOnline(self):
        if len(self.get_stories()) == 0:
            return False
        else:
            return True

    def alreadyOnline(self):
        if self.isOnline() == False:
            self.count = 0
            return False
        elif self.isOnline() == True:
            self.count = self.count + 1
        if self.count >= 2:
            self.count = 2
            return True
        else:
            return False

    def populate_stories(self):
        items = self.get_stories()
        self.walker = urwid.SimpleListWalker(items)
        self.listbox = urwid.ListBox(self.walker)
        return self.listbox

    def set_status_bar(self, msg):
        msg = '%s' % (msg.rjust(len(msg)+1))
        self.view.set_footer(urwid.AttrWrap(urwid.Text(msg), 'footer'))

    def update_ticker(self):
        self.tickers = self.get_tickers()
        if self.isOnline() == False:
            self.view.set_body(urwid.AttrWrap(self.populate_stories(), 'offline_bg'))
            self.view.set_header(header=self.offlineHeader)
            self.view.set_footer(urwid.AttrWrap(urwid.Text("You are currently offline. Please check your internet connection."), 'offline'))
        else:
            self.set_status_bar("Ticker initalised.")

    def open_story_link(self):
        url = self.listbox.get_focus()[0].story_link
        open_browser(url)

    def scroll_up(self):
        if self.listbox.focus_position - 1 in self.walker.positions():
            self.listbox.set_focus(
                self.walker.prev_position(self.listbox.focus_position)
            )

    def scroll_down(self):
        if self.listbox.focus_position + 1 in self.walker.positions():
            self.listbox.set_focus(
                self.walker.next_position(self.listbox.focus_position)
            )

    def mouse_input(self, input):
        if input[1] == self.mouse_button['left']:
            self.open_story_link()
        elif input[1] == self.mouse_button['wheel_up']:
            self.scroll_up()
        elif input[1] == self.mouse_button['wheel_down']:
            self.scroll_down()

    def keystroke(self, input):
        if input in self.keys['quit'].lower():
            raise urwid.ExitMainLoop()
        if input is self.keys['open'] or input is self.keys['tabopen']:
            self.open_story_link()
        if input is self.keys['refresh']:
            self.set_status_bar('Refreshing for new stories...')
            self.loop.draw_screen()
            self.refresh_with_new_stories()
        if input is self.keys['latest']:
            open_browser(self.link)
        if input is self.keys['scroll_up']:
            self.scroll_up()
        if input is self.keys['scroll_down']:
            self.scroll_down()
        if input is self.keys['top']:
            if self.listbox.focus_position - 1 in self.walker.positions():
                self.listbox.set_focus(self.walker.positions()[0])
        if input is self.keys['bottom']:
            if self.listbox.focus_position + 1 in self.walker.positions():
                self.listbox.set_focus(self.walker.positions()[-1])

    def handle_user_input(self, input):
        if type(input) is tuple:
            self.mouse_input(input)
        elif type(input) is str:
            self.keystroke(input)

    def refresh_with_new_stories(self):
        items = self.get_stories()
        self.alreadyOnline()
        if self.count == 0:
            self.tickers = []
            self.view.set_body(urwid.AttrWrap(self.populate_stories(), 'offline_bg'))
            self.view.set_header(header=self.offlineHeader)
            self.view.set_footer(urwid.AttrWrap(urwid.Text("You are currently offline. Please check your internet connection."), 'offline'))
        if self.count == 1:
            self.view.set_header(header=self.header)
            self.view.set_body(urwid.AttrWrap(self.populate_stories(), 'body'))
            self.update_ticker()
        else:
            self.walker[:] = items
            self.loop.draw_screen()

    def get_tickers(self):
        bbc = BBC()
        ticker_objs = bbc.get_ticker()
        return ticker_objs

    def set_latest_links(self, link):
        self.link = link

    def next_item(self, loop, *args):
        text = self.tickers
        if(not text):
            self.link = ''
            self.view.set_footer(urwid.AttrWrap(urwid.Text(""), 'body'))
        else:
            self.loop.draw_screen()
            if(self.ticker_count < len(text)):
                self.ticker_count += 1
            if(self.ticker_count == len(text)):
                self.ticker_count = 0
            if(not text[self.ticker_count].url and text[self.ticker_count].breaking == "true"):
                final_ticker = "[" + text[self.ticker_count].prompt +"] " + text[self.ticker_count].headline
                msg = '%s' % (final_ticker.rjust(len(final_ticker)+1))
                self.view.set_footer(urwid.AttrWrap(urwid.Text(msg), 'breaking'))
                self.set_latest_links(text[self.ticker_count].url)
            elif text[self.ticker_count].url and text[self.ticker_count].breaking == "true":
                final_ticker = "[" + text[self.ticker_count].prompt +"] " + text[self.ticker_count].headline
                msg = '%s' % (final_ticker.rjust(len(final_ticker)+1))
                self.view.set_footer(urwid.AttrWrap(urwid.Text(msg), 'breaking'))
                self.set_latest_links(text[self.ticker_count].url)
            else:
                self.set_status_bar("[" + text[self.ticker_count].prompt +"] " + text[self.ticker_count].headline)
                self.set_latest_links(text[self.ticker_count].url)
        self.loop.set_alarm_in(10, self.next_item)

    def _wrapped_refresh(self, loop, *args):
        online = self.isOnline()
        self.update_ticker()
        self.refresh_with_new_stories()
        ct = datetime.now().strftime('%H:%M:%S')
        if online == False:
            self.set_status_bar('You are currently offline. Please check your internet connection.')
        else:
            self.set_status_bar('Automatically updated ticker and fetched new stories at: %s' % ct)
        self.loop.set_alarm_in(200, self._wrapped_refresh)

def live():
    u = UI()
    u.run()
