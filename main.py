import time
import requests

session = requests.Session()

url = "http://api.lmc.ethancoward.dev/"

assemble_url = url + "assemble"

payload = """
INP
STA FIRST
INP
ADD FIRST
OUT
HLT

FIRST DAT 0
"""

response = session.post(assemble_url, data=payload)

# print(response.json())

state = response.json()["state"]
print(state)

step_url = url + "step"

requires_input = response.json()["next_requires_input"]

while state["pc"] != -1:
    user_input = input("Enter input: ")

    if user_input == "q":
        break
    elif user_input == "s":
        if requires_input:
            print("No input provided.")
            continue

        payload = {"state": state, "input": []}
        st = time.perf_counter()
        response = session.post(step_url, json=payload)
        print("Step time: " + str(time.perf_counter() - st))

        state = response.json()["state"]
        print(state)
        print("Output: " + str(response.json()["output"]))
        requires_input = response.json()["next_requires_input"]
    else:
        user_input = int(user_input)
        payload = {"state": state, "input": [user_input]}
        st = time.perf_counter()
        response = session.post(step_url, json=payload)
        print("Step time: " + str(time.perf_counter() - st))

        state = response.json()["state"]
        print(state)
        print("Output: " + str(response.json()["output"]))
        requires_input = response.json()["next_requires_input"]


# The code above is a simple REPL that allows you to step through the program one instruction at a time. You can enter a number to be used as input, or you can enter s to step through the program. You can also enter q to quit.
