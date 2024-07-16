# FlikIsDining Bot
a simple discord bot that shows you what's for lunch using the flikisdining api. This is a private bot for my friends and I, but feel free to use it for your own server.
this is v2 of the bot, now written in rust. The original variant is not available on GitHub.

## Commands
If inside of the primary channel (see the .env) the bot will respond to messages with `what` and `lunch`. Otherwise, must have `what lunch` continously in the message.

this means you can have fun and be like `yooyoyoyo what is the lunch for today??` and it'll respond. 

You can also add `tmr` or `tomorrow` to see the lunch for tomorrow, and this can be chained. So `tmr tmr` will show the lunch for the day after tomorrow.

### example image:
![example image](https://derock.media/r/UbYsEe.png)

You can also send `when will we have <something>` and the bot will search the next 3 weeks for any occurences of the food using a fuzzy search. 

### example image:
![example image](https://derock.media/r/YipYXB.png)

## notice
this isn't meant to be a public bot, so it's not very user friendly. this is also one of my first projects in Rust, so there's probably a lot of bad code.
