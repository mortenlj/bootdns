# action will be run in python3 container
FROM python:3@sha256:b89c8a45626cd670f81e0504084362c7e88dd432090b08f6de1b7b047fcf6c4a
# copying requirements.txt and install the action dependencies
COPY requirements.txt /requirements.txt
RUN pip install -r /requirements.txt
# script.py is the file that will contain the codes that we want to run for this action.
COPY script.py /script.py
# we will just run our script.py as our docker entrypoint by python script.py
CMD ["python", "/script.py"]
