using CSV, CategoricalArrays, DataFrames, Gadfly

if true
    df = CSV.read("dataviz/output.csv", DataFrame)
    df.selection = categorical(df.selection)
    plot(df, xgroup=:half_life, x=:selection, Geom.subplot_grid(Geom.histogram))
end
