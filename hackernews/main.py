import urwid
import subprocess
from hnapi import HN


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

    def update_with_stories(self):
        items = list()
        for story in get_top_stories():
            items.append(ItemWidget(story))
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
                url = self.listbox.get_focus()[0].story_link
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

        view = urwid.Frame(
            urwid.AttrWrap(self.update_with_stories(), 'body'),
            header=header
        )

        loop = urwid.MainLoop(view, self.palette, unhandled_input=keystroke)
        loop.screen.set_terminal_properties(colors=256)
        loop.set_alarm_in(200, self._wrapped_refresh)

        def update_footer():
            url = self.listbox.get_focus()[0].story_link
            view.set_footer(urwid.AttrWrap(urwid.Text(url), 'footer'))

        urwid.connect_signal(self.walker, 'modified', update_footer)

        try:
            loop.run()
        except KeyboardInterrupt:
            print "Keyboard interrupt received, quitting gracefully"
            raise urwid.ExitMainLoop

    def refresh(self):
        items = list()
        for story in get_top_stories():
            items.append(ItemWidget(story))

    def _wrapped_refresh(self, loop, *args):
        self.refresh()
        loop.set_alarm_in(200, self._wrapped_refresh)

if __name__ == "__main__":
    UI()
