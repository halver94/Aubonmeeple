# Scrapy

Scrapy is a small project developped in rust aiming to scrap Okkazeo,
a french 2nd hand boardgame selling website and compare its price to 
the new version from different stores in order to see if it's a good
deal or not.

That project is kind of a POC for now and has mainly been developped
on a laptop, in a hotel room, at late night, after few beers.

## Installation

At the root directory , execute the following script :

```
chmod +x ./helper/deploy.sh

./helper/deploy.sh

```

Then follow the prompt questions.

Project is divided into 2 binaries, frontend and backend.
Backend do queries and fill the DB, frontend do DB queries and serves html pages.
To compile them :
```
cargo build
```

To test your stuff, you can simply start the backend in one terminal and the frontend in another.


