# action will be run in python3 container
FROM python:3@sha256:3abe339a3bc81ffabcecf9393445594124de6420b3cfddf248c52b1115218f04
# copying requirements.txt and install the action dependencies
COPY requirements.txt /requirements.txt
RUN pip install -r /requirements.txt
# script.py is the file that will contain the codes that we want to run for this action.
COPY script.py /script.py
# we will just run our script.py as our docker entrypoint by python script.py
CMD ["python", "/script.py"]
