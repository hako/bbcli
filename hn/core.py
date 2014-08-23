import os
import urwid
import subprocess
import ConfigParser
from hnapi import HN
from datetime import datetime

_config = None

class HNStory(object):

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
    hn = HN()
    for i, story in enumerate(hn.get_top_stories()[:30]):
        yield HNStory(i, story)


def read_config():
    filename = 'pyhackernews'
    config = ConfigParser.ConfigParser()
    if os.path.exists(os.path.expanduser('~' + '/.' + filename)):
        config.read(os.path.expanduser('~' + '/.' + filename))
    elif os.path.exists ( os.path.expanduser('~' + '/.config/' + filename)):
        config.read(os.path.expanduser('~' + '/.config/' + filename))
    return config


def open_browser(function, url):
    global _config
    try:
        command = _config.get('Commands', function)
        if command:
            subprocess.Popen(
                command.replace('%URL', url).split(' '),
                stdout=open(os.devnull),
                stderr=open(os.devnull),
            )
        else:
            raise
    except:
        subprocess.Popen(
            ['python', '-m', 'webbrowser', '-t', url],
            stdout=open(os.devnull),
            stderr=open(os.devnull),
        )


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
        ('head', '', '', '', 'g7', '#f60'),
        ('body', '', '', '', 'g7', 'g66'),
        ('footer', '', '', '', 'g7', 'g55'),
        ('focus', '', '', '', 'g7', 'g55'),
        ('subtext', '', '', '', 'g38', 'g66'),
    ]

    header = [
        urwid.AttrWrap(urwid.Text(
            ' Y | Hacker News', align='left'), 'head'
        ),
        ('flow', urwid.AttrWrap(urwid.Text(
            ' ', align="right"), 'head'
        ))
    ]
    header = urwid.Columns(header)

    # defaults
    keys = {
        'quit'        : 'q',
        'open'        : 'Enter',
        'tabopen'     : 'Enter',
        'refresh'     : 'r',
        'scroll_up'   : 'k',
        'scroll_down' : 'j',
        'top'         : 'g',
        'bottom'      : 'G'
    }

    def run(self):
        self.make_screen()
        urwid.set_encoding('utf-8')
        urwid.connect_signal(self.walker, 'modified', self.update_footer)
        self.set_keys()
        try:
            self.loop.run()
        except KeyboardInterrupt:
            print "Keyboard interrupt received, quitting gracefully"
            raise urwid.ExitMainLoop

    def make_screen(self):
        self.view = urwid.Frame(
            urwid.AttrWrap(self.populate_stories(), 'body'),
            header=self.header
        )

        self.loop = urwid.MainLoop(
            self.view,
            self.palette,
            unhandled_input=self.keystroke
        )
        self.loop.screen.set_terminal_properties(colors=256)
        self.loop.set_alarm_in(600, self._wrapped_refresh)

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

    def populate_stories(self):
        items = self.get_stories()
        self.walker = urwid.SimpleListWalker(items)
        self.listbox = urwid.ListBox(self.walker)
        return self.listbox

    def set_status_bar(self, msg):
        msg = '%s' % (msg.rjust(len(msg)+1))
        self.view.set_footer(urwid.AttrWrap(urwid.Text(msg), 'footer'))

    def update_footer(self):
        url = self.listbox.get_focus()[0].story_link
        self.set_status_bar(url)

    def keystroke(self, input):
        if input in self.keys['quit'].lower():
            raise urwid.ExitMainLoop()
        if input is self.keys['open'] or input is self.keys['tabopen']:
            url = self.listbox.get_focus()[0].story_link
            function = [key for (key,value) in self.keys.items()
                       if value == input][0]
            open_browser(function, url)
        if input is self.keys['refresh']:
            self.set_status_bar('Refreshing for new stories...')
            self.loop.draw_screen()
            self.refresh_with_new_stories()
        if input is self.keys['scroll_up']:
            if self.listbox.focus_position - 1 in self.walker.positions():
                self.listbox.set_focus(
                    self.walker.prev_position(self.listbox.focus_position)
                )
        if input is self.keys['scroll_down']:
            if self.listbox.focus_position + 1 in self.walker.positions():
                self.listbox.set_focus(
                    self.walker.next_position(self.listbox.focus_position)
                )
        if input is self.keys['top']:
            if self.listbox.focus_position - 1 in self.walker.positions():
                self.listbox.set_focus(self.walker.positions()[0])
        if input is self.keys['bottom']:
            if self.listbox.focus_position + 1 in self.walker.positions():
                self.listbox.set_focus(self.walker.positions()[-1])

    def refresh_with_new_stories(self):
        items = self.get_stories()
        self.walker[:] = items
        self.loop.draw_screen()

    def _wrapped_refresh(self, loop, *args):
        self.refresh_with_new_stories()
        ct = datetime.now().strftime('%H:%M:%S')
        self.set_status_bar('Automatically fetched new stories at: %s' % ct)
        self.loop.set_alarm_in(600, self._wrapped_refresh)


def live():
    u = UI()
    u.run()
