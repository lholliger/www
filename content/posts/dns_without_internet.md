---
title: DNS Without the Internet
description: Sending a DNS query using USPS and 1.1.1.1
date: 2019-08-08
tags:
    - dns
    - cloudflare
---

# Why

Recently someone sent a DNS query [over a telegram](https://twitter.com/jgrahamc/status/1144272344803946496), so I wondered, what other ways can I send DNS request without using the internet to send them? On the Cloudflare website there’s a section thats called [Fun with DNS](http://web.archive.org/web/20190325132350/https://developers.cloudflare.com/1.1.1.1/fun-stuff/), and it includes ways like Google Sheets, SMS, Telegram (not a literal telegram), Tor, Twitter, and Email. But noticably, all of these methods required the internet to be sent. So I decided to try to send DNS queries using non-internet mediums.

# The Solution

When I first started thinking about this project, I looked in the comments of the twitter post above, someone asked whether you could send DNS requests using postcards, and they got a reply saying they could [as long as it wasn’t a sealed envelope](https://twitter.com/ppog_penguin/status/1144792854947962882). But, sending an envelope is easier, more secure, and allows for me to type my letter, so I decided that I should try using mail.

# Sending a Request

I went to type up a document to mail for a request. But I knew I couldn’t just send A holliger.me, because on its own, it doesn’t make sense, and theres nothing fun in just sending two words, so I wrote some more. For the letter, I decided to write something like this: I wrote up a letter and it came out something like this:

> Hello,
>
>I’ve heard that 1.1.1.1 supports many different ways of sending requests, am I able to request using DNS over USPS?
>
>If so, I would like to know the records for the following:
>
>TXT holliger.me
>
>Thanks,
>
>Lukas Holliger
>
>[my address]

I then folded it, wrote the Cloudflare Texas address, wrote my address for return, then I put a stamp on it and sent it off.

![](/assets/images/dns_without_internet/cf-letter-2048x1536.jpg)

I also decided to add something fun to the TXT of holliger.me, saying something like this:

> Hello there, does Cloudflare support Merch over USPS? My systems support Mens XL 😛

# A Reply

I received some confirmation about my letter being received when I saw some traffic on this blog, and I also received a message from a friend showing me a picture of my letter. After a few weeks of waiting, a box arrived at my house, addressed from Cloudflare Inc in Texas. Upon opening the box, I was greeted with some packing paper and this sheet and a shirt!

![](/assets/images/dns_without_internet/cf-reply.jpg)