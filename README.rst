pyhackernews
============

mimicking HN look and feel in terminal


HN in terminal with tmux:

.. image:: http://i.imgur.com/890LAsb.png

instalation and usage:
======================

``pip install pyhackernews``

and then in terminal:

``hn``

configuration:
==============

Custom keybindings can be defined in either:

``$HOME/.pyhackernews``

Or:

``$HOME/.config/pyhackernews``


Like so:

::

  [Keys]
  quit = q
  open = o
  tabopen = O
  refresh = r
  scroll_up = k
  scroll_down = j
  bottom = G

  [Commands]
  ; %URL is a placeholder for where the actual URL will be inserted.
  ; Remove these if unused.

  open = dwbremote :open %URL
  tabopen = dwbremote :tabopen %URL
