# aiapi

AI API is an interface on top of AI API providers.

Add support for multiple API providers inside your application.

## Why Not Run AI Locally

I think running AI locally is a nice idea that is unlikely to take off.
Most people will want to use the best AI models that are available.
However, this means that the hardware requirements are too high.
In most cases, running AI in the cloud will be orders of magnitude cheaper than running it locally.
For example, running DeepSeek R1 requires a $2000 server while the server still only does around 2-3 tokens per second.
Conversely, running the same model in the cloud will cost only a few cents per million tokens.
Assuming you would use about 10 000 tokens per day, the cost would still only be around $4 per year.
With the server, you would probably write it down over say 10 years, so the cost for depreciation alone would be $200 per year.
That's not even including the cost of electricity and the maintenance costs.
And the fact that hardware is getting more efficient all the time.

So that's why I expect that most people will run AI in the cloud.
Luckily, there are nowadays many providers.
This library aims to make it easy to easily built on top of these providers.
