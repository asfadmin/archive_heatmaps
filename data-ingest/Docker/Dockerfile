
FROM continuumio/miniconda3

# Set working dir
WORKDIR /data_ingest

# Copy all python files over
COPY ${PWD}/*.py .
COPY ${PWD}/Docker/env.yml .
COPY ${PWD}/.env .

RUN apt update
RUN apt install libgdal-dev \
                build-essential \
                postgis -y 

RUN conda env create -f env.yml

SHELL ["conda", "run", "-n", "data-ingest", "/bin/bash", "-c"]

CMD ["conda", "run", "--no-capture-output", "-n", "data-ingest", "python3", "ingest.py"]


