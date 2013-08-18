import urwid
import itertools
import subprocess
from bs4 import BeautifulSoup
from clint.textui import colored, indent, puts
from urllib2 import Request, urlopen, URLError


class HNSubtext(object):

    def __init__(self, item):
        self.item = item

    @property
    def subtext(self):
        return self.item.text

    @property
    def points(self):
        return self.item.span.text

    @property
    def user(self):
        return self.item.a.text


class HNStory(object):

    def __init__(self, index, item):
        self.index = index + 1
        self.item = item

    @property
    def number(self):
        index = str(self.index)
        if len(index) == 1:
            return ''.join((' ', index))
        return self.index

    @property
    def title(self):
        return self.item.text

    @property
    def href(self):
        return self.item.a['href']


def get_hackernews_source():
    url = 'https://news.ycombinator.com/'
    req = Request(url)
    try:
        with indent(4, (' >')):
            puts(colored.cyan('Fetching stories ...'))
        response = urlopen(req)
    except URLError as e:
        if hasattr(e, 'reason'):
            print 'We failed to reach a server.'
            print 'Reason: ', e.reason
        elif hasattr(e, 'code'):
            print 'The server couldn\'t fulfill the request.'
            print 'Error code: ', e.code
    soup = BeautifulSoup(response)
    return soup

page_source = get_hackernews_source()


def get_stories_titles():
    titles = page_source.select('td.title')
    for i, s in enumerate(itertools.islice(titles[1:], 0, None, 2)):
        if not s.text == 'More':
            yield HNStory(i, s)


def get_stories_subtexts():
    subtexts = page_source.select('td.subtext')
    for s in subtexts:
        yield HNSubtext(s)


def open_browser(url):
    subprocess.Popen(
        ['xdg-open', url], stdout=subprocess.PIPE, stderr=subprocess.PIPE
    )


class ItemWidget(urwid.WidgetWrap):

    def __init__(self, story_title, story_subtext):
        self.href = story_title.href
        story_title = urwid.AttrWrap(urwid.Text(
            '%s. %s' % (story_title.number, story_title.title)),
            'body', 'focus'
        )
        story_subtext = urwid.AttrWrap(urwid.Text(
            '    %s' % story_subtext.subtext),
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

    def update_with_stories(self, titles, subtexts):
        items = list()
        for t, st in zip(titles, subtexts):
            items.append(ItemWidget(t, st))
        self.walker = urwid.SimpleListWalker(items)
        self.listbox = urwid.ListBox(self.walker)
        return self.listbox

    def __init__(self):
        header = [
            urwid.AttrWrap(urwid.Text(
                ' Y | Hacker News', align='left'), 'head'
            ),
            ('flow', urwid.AttrWrap(urwid.Text(
                ' ', align="right"), 'head'
            ))
        ]
        header = urwid.Columns(header)

        def keystroke(input):
            if input in ('q', 'Q'):
                raise urwid.ExitMainLoop()
            if input is 'enter':
                url = self.listbox.get_focus()[0].href
                open_browser(url)
            if input is 'r':
                import threading
                view.set_footer(urwid.AttrWrap(urwid.Text(
                    'refreshing for new stories...'), 'footer'
                ))
                threading.Thread(target=self.refresh).start()
            if input is 'k':
                if self.listbox.focus_position - 1 in self.walker.positions():
                    self.listbox.set_focus(
                        self.walker.prev_position(self.listbox.focus_position)
                    )
            if input is 'j':
                if self.listbox.focus_position + 1 in self.walker.positions():
                    self.listbox.set_focus(
                        self.walker.next_position(self.listbox.focus_position)
                    )
            if input is 'g':
                if self.listbox.focus_position - 1 in self.walker.positions():
                    self.listbox.set_focus(self.walker.positions()[0])
            if input is 'G':
                if self.listbox.focus_position + 1 in self.walker.positions():
                    self.listbox.set_focus(self.walker.positions()[-1])

        titles = get_stories_titles()
        subtexts = get_stories_subtexts()

        view = urwid.Frame(
            urwid.AttrWrap(self.update_with_stories(titles, subtexts), 'body'),
            header=header
        )

        loop = urwid.MainLoop(view, self.palette, unhandled_input=keystroke)
        loop.screen.set_terminal_properties(colors=256)
        loop.set_alarm_in(200, self._wrapped_refresh)

        def update_footer():
            url = self.listbox.get_focus()[0].href
            view.set_footer(urwid.AttrWrap(urwid.Text(url), 'footer'))

        urwid.connect_signal(self.walker, 'modified', update_footer)

        try:
            loop.run()
        except KeyboardInterrupt:
            print "Keyboard interrupt received, quitting gracefully"
            raise urwid.ExitMainLoop

    def refresh(self):
        titles = get_stories_titles()
        subtexts = get_stories_subtexts()
        items = list()
        for t, st in zip(titles, subtexts):
            items.append(ItemWidget(t, st))

    def _wrapped_refresh(self, loop, *args):
        self.refresh()
        loop.set_alarm_in(200, self._wrapped_refresh)

if __name__ == "__main__":
    UI()
