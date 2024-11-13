---
title: ECE1100 Making door opening easier
description: Using some cheap HID scanners to not use a key
date: 2024-11-12
tags:
    - 3d-printing
    - ece1100
---

# Project Overview
For my building, I've been dealing with the inconvenience of juggling three nearly identical keys - one for the front door, one for various closets, and one for my actual room door. Despite their similar appearances, these keys are completely distinct with no overlap in functionality. After some research, I decided to explore creating a DIY card access system using relatively inexpensive components, some of which I had on hand:

- HID Omnikey RFID reader
- Arduino Uno or Raspberry Pi (dealing with how Omnikeys work is difficult)
- Servo motor mechanism
- 3D printed housing
- Basic electronic components (wiring, resistors, etc)

The basic concept is to mount Omnikey reader (which can detect Buzzcards), then trigger a servo mechanism to physically open the door by pushing the handle. This would eliminate the need to carry multiple keys while potentially adding convenient features like temporary access for different people.

# Some Backstory

A few months ago a friend posted a link to some clearance Omnikey readers (pictured below), where these could be picked up for under $10 a piece, when they retail for about $65 on Amazon. When I first got them I expected them to work like other barcode or magstripe readers where they just dump what they read over the keyboard, but instead this works as a Smart Card reader. At first this seemed like a drawback, but I quickly learned how this could be used for more detailed tasks.

![Omnikey reader](https://m.media-amazon.com/images/I/61P7x5wTziL.jpg)

I found some [old code](https://github.com/RoboJackets/apiary-nfc-reader/blob/master/card_reader_server.py) to figure out the commands, then threw together some Javascript to read GTIDs. I eventually morphed this into a scanner for parties (a writeup on that coming soon), which was able to test against different student IDs (think GSU, KSU, uGA, etc) and only accept Buzzcards. After this, I realized that I could make a good door scanner out of this, or possibly even my own vending machine system since my building has one. However, progress has been slow.

Here is an old version of the scanner I made, unfortunately I cannot include the whole video as it has GTID information on it.

![](/assets/images/ece1100_showcase/scanner.png)

# Current Progress

So far, I've made initial progress by acquiring and testing an HID Omnikey reader to verify its basic functionality. I've also worked on prototyping the Arduino code needed for servo control and successfully tested reading various cards using the Omnikey system. Currently, I'm working through understanding the optimal positions for servo mounting as well as how to keep power while the door is opened.

Looking ahead, the next major phase of the project will involve building a complete prototype assembly, conducting tests with various types of cards to make sure only Buzzcards work and that other cards cannot spoof access, developing the necessary control software, and creating documentation for installation since it seems like it may be useful for other people.
