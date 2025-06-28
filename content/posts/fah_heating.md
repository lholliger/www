---
title: Folding at Home Heater
description: When Folding at Home produced a little too much heat
date: 2025-03-10
tags:
    - hardware
---

In November 2024, my building was having issues heating, so I decided it would be a good idea to pull out my old desktop and do some Folding@Home in order to get some heat. At first this simply started my personal gaming rig's RTX 3080. However, this wouldn't work too well in the long term since this is an SFF build that has very little ability for heat dissipation. To make matters worse, I joined a team with a friend from Apple who has a 3090 and an EPYC doing some folding, so I had to do what I could to get in first place!

![Folding at Home UI](/assets/images/fah_heating/fah_gui.png)

So now I began scowering through Facebook Marketplace to find some cheap GPUs to use in my room heater. Eventually I came across an old NVIDIA CMP 90HX card, which was just RTX 3080 dies used for mining, or so I thought. After contacting the buyer and agreeing on a price, we met up and he let me know he had a second identical card and I could get them together for a lower price, so of course I agreed and got both cards. After this is where all the problems begin...

![CMP 90 HX GPUs](/assets/images/fah_heating/75461935704__BCAE8B52-72AC-4650-B21B-CC21C47E1FCF.heic)

I knew these cards had no graphics output, and since I was only going to use them for compute tasks I had no use for that feature. What I didn't know however is they had many internal functions disabled. When I first installed these cards and booted them up, it looked like everything was fine and going to work. I had an old mining riser sitting around from when I did it years ago so I plugged everything in and booted up the computer to get these cards tested. At first, I decided to do what these cards were best at: mining. They seemed to perform that job perfectly, so next was figuring out how to get them to fold.

![brat PC](/assets/images/fah_heating/IMG_1834.heic)

When I spun up Folding@Home it all looked to be going well, but then I noticed these cards were highly underperforming and underpulling power. Instead of some 6 million points per day, they were producing around 200k. For something that should be a 3080, this seems highly unusual. After some digging, it turns out these cards have almost all of their FP32 units disabled and only really having INT8 performance. When looking into seeing if this is a driver patch, I came up dry. I ended up [creating an issue](https://github.com/dartraiden/NVIDIA-patcher/issues/192) to see if there was any way around it, but it seems for now thats a dead end.

So for now, I was somewhat stuck. These cards couldn't be used for folding, and I didn't really want to do much mining on them. So I did the next best thing: I listed the other GPUS back on Marketplace and began the search for more GPUs. Rather quickly, I found a guy selling 3080's. Not just any random ones, Founders Edition cards. I contacted him of what price would be good to buy the rest of his lot, and we agreed and met. I tested all of the cards in Passmark since they were formerly used for mining and all had all functions working. So I packed them in my car and took them home.

![3080 pile](/assets/images/fah_heating/3080_pile.png)

Now I really had no use for 5 3080 GPUs, I only wanted two. So I sold one at cost to my friend and another to family, then sold another for a small markup elsewhere. Now I was left with two rather cheap 3080 FE cards! I put these into the system and got to folding. Everything worked just as it did with the single GPU; they folded at around 6m points each and very nicely heated my place. To keep the power supply (750w HP server PSU) quiet I underpowered the GPUS to 250w each since they were providing more than enough heat and they were providing the most efficient point counts at that level (on Ampere-series cards, the last 20-30% of power only gets like 5% performance, so dropping lower is almost unnoticable and you might even gain performance due to easier cooling).

![3080 folding rig](/assets/images/fah_heating/IMG_1925.heic)

Everything here seemed great, I got a bunch of points, rocketed into first place, got my place warm, and got to feel a little good that I was doing research with my compute. After a few weeks of on-and-off folding, some unexpected issues started to appear however. When I was off studying in the library someone asked in my building's group chat where a burning smell was coming from. It seemed like something got burned cooking or it was some sort of gas. I thought it was nothing, since I opened up my folding page to see if there were any issues and everything was operating as normal. When I got back to my place I could smell the horrible electic-ish scent but I thought it was just some food someone burned or some animal, so after some days of keeping my window open everything seemed fine.

Fast forward to a few nights later. It's starting to get a bit warmer so I need to do less folding, but this night was especially cold. So I pulled out my phone and fired up the folding process. Very quickly however I smelled that old burning smell again. I quickly jumped out of my bed and looked at the rig to notice the PSU breakout board smoking.

![Crispy connectors](/assets/images/fah_heating/IMG_2896.heic)

I pulled all the plugs and opened up the window to air out the place. I inspected the cables and saw that the cables started to melt into the breakout board and some of the connectors were exposed. I guess there was some resistace in one of the cables causing one to melt, which cascaded into a whole series of melting connectors. Thankfully there was nothing damaged except for this connector; the GPUs, PSU, and the building were all fine.

Since it was reaching spring time, this marked the end of any folding operations. I later went into our makerspace to see what the wire resistances ended up becoming and I got to this chart:

![Wire Resistances](/assets/images/fah_heating/IMG_7192.png)

That's a lot more resistance than a wire should have! I used these cables for years and never had any issues but I guess with some small way I connected it or just for how long I've used them they began to fail. I haven't purchased the parts again since I haven't needed any more heat, and I'm also a little afaid to try the project again. I got some feedback that I should get bigger 12VHPWR splitters that use 3 or 4 4-pins instead of the standard two that came with the 3080s. If I do this project again in the future, I'll be sure to pick those up.

Overall I learned a lot with this project. I learned how mining GPUs remain e-waste useful only for mining and how to melt GPU connectors. This definitely won't be the last time I do something like this, however for the forseeable future I'll be paying for my own power instead of it being included in rent, so I'll probably stick to less heat or, if there's central heat, just using that.