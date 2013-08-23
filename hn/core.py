import urwid
import subprocess
from hnapi import HN
from datetime import datetime


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


def open_browser(url):
    subprocess.Popen(
        ['xdg-open', url], stdout=subprocess.PIPE, stderr=subprocess.PIPE
    )


class ItemWidget(urwid.WidgetWrap):

    def __init__(self, s):
        self.story_link = s.story_link
        story_title = urwid.AttrWrap(urwid.Text(
            u'%s. %s' % (s.story_number, s.story_title)),
            'body', 'focus'
        )
        story_subtext = urwid.AttrWrap(urwid.Text(
            u'    %s' % (s.story_subtext)),
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

    def run(self):
        self.make_screen()
        urwid.connect_signal(self.walker, 'modified', self.update_footer)
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

    def update_footer(self):
        url = self.listbox.get_focus()[0].story_link
        self.view.set_footer(urwid.AttrWrap(urwid.Text(' %s' % url), 'footer'))

    def keystroke(self, input):
        if input in ('q', 'Q'):
            raise urwid.ExitMainLoop()
        if input is 'enter':
            url = self.listbox.get_focus()[0].story_link
            open_browser(url)
        if input is 'r':
            self.refresh_with_new_stories()
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

    def refresh_with_new_stories(self):
        items = self.get_stories()
        self.walker[:] = items
        self.loop.draw_screen()

    def _wrapped_refresh(self, loop, *args):
        self.refresh_with_new_stories()
        ct = datetime.now().strftime('%H:%M:%S')
        self.view.set_footer(urwid.AttrWrap(urwid.Text(
            ' Automatically fetched new stories at: %s' % ct), 'footer'
        ))
        self.loop.set_alarm_in(600, self._wrapped_refresh)


def live():
    u = UI()
    u.run()
