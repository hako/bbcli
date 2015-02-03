bbcli
============

Browse BBC News through the command line. (based on pyhackernews)

.. image:: https://cloud.githubusercontent.com/assets/2040416/6029751/a176a20a-abea-11e4-8be4-ba435b3b48c0.gif

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
