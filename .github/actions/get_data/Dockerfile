# action will be run in python3 container
FROM python:3@sha256:aaf6d3c4576a462fb335f476bed251511f2f1e61ca8e8e97e9e197bc92a7a1ee
# copying requirements.txt and install the action dependencies
COPY requirements.txt /requirements.txt
RUN pip install -r /requirements.txt
# script.py is the file that will contain the codes that we want to run for this action.
COPY script.py /script.py
# we will just run our script.py as our docker entrypoint by python script.py
CMD ["python", "/script.py"]
