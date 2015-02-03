bbcli
============

Browse BBC News through the command line. (based on pyhackernews)

instalation and usage:
======================

``pip install bbcli``

and then in terminal:

``bbcli``

configuration:
==============

Custom keybindings can be defined in either:

``$HOME/.bbcli``

Or:

``$HOME/.config/bbcli``


Like so:

::

  [Keys]
  quit = q
  open = w
  tabopen = O
  refresh = r
  latest = l
  scroll_up = k
  scroll_down = j
  bottom = G

  [Commands]
  ; %URL is a placeholder for where the actual URL will be inserted.
  ; Remove these if unused.

  open = dwbremote :open %URL
  tabopen = dwbremote :tabopen %URL

credits
=======
Dan Claudiu Pop and Chase Franklin for pyhackernews.