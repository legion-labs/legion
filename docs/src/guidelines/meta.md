# The meta guidelines

## **- META-001** - Everyone is allowed to contributes to the guidelines!

Guidelines the same as code should not be fixed and rigid to change by anyone. The goal of maintaining healthy guidelines is to have everyone collaborate in curating and improving them. If controversial make sure to have a discussion and be open minded about it!

## **- META-002** - Guidelines naming and state

Guidelines are named as follow: CATEGORY-XXX[-STATE]. Where CATEGORY is the category name, XXX is a 3 digit number starting from 001. STATE denotes the state of the guideline, can be either BETA, meaning it is being tested to see if it provides actionable information or DEPRECATED meaning it is no longer in use...

## **- META-003** - Provides links to specific guidelines in reviews

Whenever is necessary, provide the link to a specific guideline when doing a review. If the guideline is deemed to ambiguous by the reviewee, don't debate it in the review, and follow the reviewer reading/clarification. Don't hesitate to open a PR to improve on the guideline clarity.

## **- META-004** - Start somewhere and iterate

We believe in iterative improvement, no idea, code, architecture is at its best form at first! We start from somewhere and improve, strive to improve the work of others. Also, there is no shame in going the wrong direction, it's part of the iterative process. If anything takes more than a week of discussion, it will not be resolved by discussing it more, but by rather writing some code and seeing where it goes.

## **- META-005** - Assume good intent

Always assume good intent when interacting with others, be open minded, willing to try things, and +1 other people's ideas instead of shutting them down, it's part of the Legion Labs company culture as well.

## **- META-006** - Running >> Reading >> Debugging >> Writing

The golden rule of programming at Legion Labs, writing code is usually the act of one individual and in the end the least frequent activity a programmer does; debugging can involve more people and more time, but most people spend time reading code, their own, others, even when debugging. But in the end, the code was meant to be executed, and we should never prioritize anything over the execution properties of the code: correctness, proper error handling, stability and speed when it matters. When looking at it the other way around, it is rather crazy to skip on some characters and reduce code readability. We should strive to lower the cognitive energy put in understanding the code and debugging it, as long we don’t compromise its execution properties.

## **- META-007** - Be mindful of your local workflow and others

Think of the user's flow first but be mindful of the your local workflow and others', it generally translates to iteration times.
If you are a sound programmer, requiring everyone to run with sound is not mindful of the local workflows of others. The same goes if you are a 3D programmers and you require everyone to load the highest mips of all textures all the time because the game looks better that way (although it's true, but also slow...):

- Have ways for people to test your code (unit tests, integration tests part of the default set of tests)
- Have a way to turn a system off in runtime whenever possible (the switch can be through the whole lifetime of the process as handling as doing otherwise might complicate things further)
