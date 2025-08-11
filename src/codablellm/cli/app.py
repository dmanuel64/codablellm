from typer import Typer


app = Typer()

@app.command()
def command(a):
    print(locals())