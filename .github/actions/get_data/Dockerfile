# action will be run in python3 container
FROM python:3@sha256:ede46d7778a7ebac1e2f5253cdb9b59d3928c736b280636f68dcc60250619a03
# copying requirements.txt and install the action dependencies
COPY requirements.txt /requirements.txt
RUN pip install -r /requirements.txt
# script.py is the file that will contain the codes that we want to run for this action.
COPY script.py /script.py
# we will just run our script.py as our docker entrypoint by python script.py
CMD ["python", "/script.py"]
